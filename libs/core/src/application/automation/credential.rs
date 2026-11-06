use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        credential::{
            CreateCredentialCommand, Credential, CredentialError, CredentialOrigin,
            CredentialStatus, MAX_CREDENTIALS_PER_ORGANIZATION, UpdateCredentialCommand,
            assemble_fields, check_provider_urls, destination_moved, ensure_origin_allowed,
            is_internal_secret, require_secrets_after_a_move, validate_description, validate_name,
        },
        credential_kind::{CredentialKind, credential_kind},
        ports::CredentialRepository,
        secret::SealedSecret,
    },
    infrastructure::automation::webhook::{
        address_policy::PrivateNetworkAccess, secret::SecretCipher,
    },
};

const NAME_CONSTRAINT: &str = "uq_automation_credential_name";

fn kind_of(kind: &str) -> Result<&'static CredentialKind, CoreError> {
    credential_kind(kind).ok_or_else(|| {
        CredentialError::UnknownKind {
            kind: kind.to_owned(),
        }
        .into()
    })
}

fn name_taken(error: CoreError) -> CoreError {
    match error {
        CoreError::Conflict(constraint) if constraint == NAME_CONSTRAINT => {
            CredentialError::NameTaken.into()
        }
        other => other,
    }
}

pub(super) fn seal_object(
    cipher: &SecretCipher,
    object: &Map<String, Value>,
) -> Result<SealedSecret, CoreError> {
    let plaintext = serde_json::to_vec(object)
        .map_err(|e| CoreError::Internal(format!("credential data cannot be serialized: {e}")))?;
    cipher.seal(&plaintext)
}

fn parse_object(plaintext: &[u8]) -> Result<Map<String, Value>, CoreError> {
    match serde_json::from_slice(plaintext) {
        Ok(Value::Object(object)) => Ok(object),
        _ => Err(CoreError::Internal(
            "a sealed credential does not hold a JSON object".to_owned(),
        )),
    }
}

pub(super) fn open_object(
    cipher: &SecretCipher,
    sealed: &SealedSecret,
) -> Result<Map<String, Value>, CoreError> {
    parse_object(&cipher.open(sealed)?)
}

fn split_unvalidated(
    kind: &CredentialKind,
    object: Map<String, Value>,
) -> (Map<String, Value>, Map<String, Value>) {
    let mut readable = Map::new();
    let mut secrets = Map::new();
    for (name, value) in object {
        match kind.field(&name) {
            Some(field) if !field.secret => readable.insert(name, value),
            _ => secrets.insert(name, value),
        };
    }
    (readable, secrets)
}

pub(super) fn secret_names(secrets: &Map<String, Value>) -> Vec<String> {
    secrets.keys().cloned().collect()
}

async fn split_legacy<R: CredentialRepository>(
    cipher: Option<&SecretCipher>,
    repository: &mut R,
    org_id: OrganizationId,
) -> Result<(), CoreError> {
    let Some(cipher) = cipher else {
        return Ok(());
    };
    for (credential, sealed) in repository.list_unsplit(org_id).await? {
        let Some(kind) = credential_kind(&credential.kind) else {
            continue;
        };
        let opened = match open_object(cipher, &sealed) {
            Ok(opened) => opened,
            Err(error) => {
                tracing::warn!(
                    org_id = %org_id.0,
                    credential_id = %credential.id,
                    %error,
                    "a stored credential could not be opened and was left unsplit"
                );
                continue;
            }
        };
        let (readable, secrets) = split_unvalidated(kind, opened);
        let resealed = seal_object(cipher, &secrets)?;
        let split = Credential {
            data: readable,
            secrets_set: secret_names(&secrets),
            fields_split: true,
            ..credential
        };
        repository.store_split(org_id, &split, &resealed).await?;
    }
    Ok(())
}

impl MestierUseCase {
    pub(super) fn provider_urls_must_be_https(&self) -> bool {
        self.oauth_access == Some(PrivateNetworkAccess::Denied)
    }

    pub(super) fn require_cipher(&self) -> Result<Arc<SecretCipher>, CoreError> {
        self.cipher.clone().ok_or_else(|| {
            CoreError::Conflict(
                "no automation secret key configured: credentials cannot be sealed".to_owned(),
            )
        })
    }

    #[transactional(credential)]
    pub async fn create_credential(
        &self,
        command: CreateCredentialCommand,
    ) -> Result<(Credential, Option<Vec<u8>>), CoreError> {
        let cipher = self.require_cipher()?;
        let kind = kind_of(&command.kind)?;
        ensure_origin_allowed(kind, command.origin)?;
        let name = validate_name(&command.name)?;
        let description = validate_description(command.description.as_deref().unwrap_or(""))?;

        let (readable, plaintext, secrets_set, revealed) = match command.origin {
            CredentialOrigin::Supplied => {
                let data = command.data.ok_or(CredentialError::MissingData)?;
                let set = assemble_fields(kind, &Map::new(), &data)?;
                check_provider_urls(kind, &set.readable, self.provider_urls_must_be_https())?;
                let plaintext = serde_json::to_vec(&set.secrets).map_err(|e| {
                    CoreError::Internal(format!("credential data cannot be serialized: {e}"))
                })?;
                let names = set.secret_names();
                (set.readable, plaintext, names, false)
            }
            CredentialOrigin::Generated => {
                let names = kind.secret_names().into_iter().map(str::to_owned).collect();
                (Map::new(), cipher.generate_secret()?, names, true)
            }
        };
        let sealed = cipher.seal(&plaintext)?;

        let now = Utc::now();
        let credential = Credential {
            id: generate_uuid_v7(),
            org_id: command.org_id,
            kind: command.kind,
            name,
            description,
            origin: command.origin,
            status: Credential::initial_status(kind.kind),
            account: None,
            expires_at: command.expires_at,
            data: readable,
            secrets_set,
            fields_split: true,
            created_at: now,
            updated_at: now,
        };

        let mut repository = credential_repository;
        if repository
            .count_locking_organization(command.org_id)
            .await?
            >= MAX_CREDENTIALS_PER_ORGANIZATION
        {
            return Err(CredentialError::TooManyCredentials.into());
        }
        let inserted = repository
            .insert(&credential, &sealed)
            .await
            .map_err(name_taken)?;
        Ok((inserted, revealed.then_some(plaintext)))
    }

    #[transactional(credential)]
    pub async fn open_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(Credential, Vec<u8>), CoreError> {
        let cipher = self.require_cipher()?;
        let mut repository = credential_repository;
        let (credential, sealed) = repository
            .find_sealed_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let plaintext = cipher.open(&sealed)?;

        match credential.origin {
            CredentialOrigin::Generated => Ok((credential, plaintext)),
            CredentialOrigin::Supplied => {
                let mut merged = credential.data.clone();
                merged.extend(parse_object(&plaintext)?);
                let bytes = serde_json::to_vec(&Value::Object(merged)).map_err(|e| {
                    CoreError::Internal(format!("credential data cannot be serialized: {e}"))
                })?;
                Ok((credential, bytes))
            }
        }
    }

    pub async fn open_credential_for_run(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(Credential, Vec<u8>), String> {
        match self.open_credential(org_id, id).await {
            Ok(opened) => Ok(opened),
            Err(CoreError::NotFound) => Err(self.missing_credential_message(org_id, id).await),
            Err(error) => Err(format!("the credential is unusable: {error}")),
        }
    }

    pub async fn missing_credential_message(&self, org_id: OrganizationId, id: Uuid) -> String {
        let name = self
            .deleted_credential_name(org_id, id)
            .await
            .ok()
            .flatten();
        match name {
            Some(name) => format!("credential `{name}` no longer exists"),
            None => format!("credential `{id}` no longer exists"),
        }
    }

    #[transactional(credential)]
    pub async fn deleted_credential_name(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<String>, CoreError> {
        let mut repository = credential_repository;
        repository.deleted_name(org_id, id).await
    }

    #[transactional(credential)]
    pub async fn list_credentials(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<Credential>, CoreError> {
        let mut repository = credential_repository;
        split_legacy(self.cipher.as_deref(), &mut repository, org_id).await?;
        repository.list_by_organization(org_id).await
    }

    #[transactional(credential)]
    pub async fn find_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Credential>, CoreError> {
        let mut repository = credential_repository;
        split_legacy(self.cipher.as_deref(), &mut repository, org_id).await?;
        repository.find_by_id(org_id, id).await
    }

    #[transactional(credential)]
    pub async fn credential_usage(
        &self,
        org_id: OrganizationId,
    ) -> Result<HashMap<Uuid, i64>, CoreError> {
        let mut repository = credential_repository;
        Ok(repository.usage_counts(org_id).await?.into_iter().collect())
    }

    #[transactional(credential)]
    pub async fn update_credential(
        &self,
        command: UpdateCredentialCommand,
    ) -> Result<Credential, CoreError> {
        let mut repository = credential_repository;
        split_legacy(self.cipher.as_deref(), &mut repository, command.org_id).await?;
        let (existing, sealed) = repository
            .find_sealed_for_update(command.org_id, command.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let kind = kind_of(&existing.kind)?;

        let name = match &command.name {
            Some(name) => validate_name(name)?,
            None => existing.name.clone(),
        };
        let description = match &command.description {
            Some(description) => validate_description(description)?,
            None => existing.description.clone(),
        };
        let expires_at = match command.expires_at {
            Some(expiry) => expiry,
            None => existing.expires_at,
        };

        let mut updated = Credential {
            name,
            description,
            expires_at,
            ..existing
        };

        let resealed = match command.data {
            None => None,
            Some(data) => {
                if updated.origin == CredentialOrigin::Generated {
                    return Err(CredentialError::GeneratedHoldsNoFields.into());
                }
                let cipher = self.require_cipher()?;
                let opened = open_object(&cipher, &sealed)?;
                let mut stored = updated.data.clone();
                let mut internal = Map::new();
                for (name, value) in opened {
                    if is_internal_secret(&name) {
                        internal.insert(name, value);
                    } else {
                        stored.insert(name, value);
                    }
                }
                let set = assemble_fields(kind, &stored, &data)?;
                check_provider_urls(kind, &set.readable, self.provider_urls_must_be_https())?;
                let moved = destination_moved(kind, &updated.data, &set.readable);
                if moved {
                    require_secrets_after_a_move(kind, &data)?;
                }
                let replaced_a_secret = data
                    .as_object()
                    .is_some_and(|patch| patch.keys().any(|name| set.secrets.contains_key(name)));

                let mut secrets = set.secrets;
                if moved {
                    internal.clear();
                    updated.account = None;
                    updated.status = Credential::initial_status(kind.kind);
                }
                secrets.extend(internal);
                updated.data = set.readable;
                updated.secrets_set = secret_names(&secrets);
                if !moved && replaced_a_secret && updated.status == CredentialStatus::Expired {
                    updated.status = CredentialStatus::Valid;
                    if command.expires_at.is_none()
                        && updated
                            .expires_at
                            .is_some_and(|expiry| expiry <= Utc::now())
                    {
                        updated.expires_at = None;
                    }
                }
                Some(seal_object(&cipher, &secrets)?)
            }
        };

        repository
            .update(command.org_id, &updated, resealed.as_ref())
            .await
            .map_err(name_taken)
    }

    #[transactional(credential)]
    pub async fn delete_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let mut credentials = credential_repository;

        credentials
            .find_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;

        credentials.delete(org_id, id).await
    }

    #[transactional(credential)]
    pub async fn rotate_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(Credential, Vec<u8>), CoreError> {
        let mut repository = credential_repository;
        let existing = repository
            .find_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        existing.origin.ensure_rotatable()?;

        let cipher = self.require_cipher()?;
        let plaintext = cipher.generate_secret()?;
        let sealed = cipher.seal(&plaintext)?;

        let updated = repository.update(org_id, &existing, Some(&sealed)).await?;
        Ok((updated, plaintext))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::credential::CredentialOrigin;
    use crate::domain::automation::secret::SealedSecret;
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, Edge, Graph, PlacedConnector, PlacedTrigger, TriggerKind,
    };
    use crate::infrastructure::automation::webhook::{
        address_policy::PrivateNetworkAccess, secret::SecretCipher,
    };
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn use_case(pool: PgPool) -> MestierUseCase {
        let mut usecase = MestierUseCase::new(pool, default_authorizer(), EventHub::new());
        usecase.cipher = Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap()));
        usecase
    }

    fn use_case_without_cipher(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn graph_using(credential_id: Uuid) -> Graph {
        Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "http.request".to_string(),
                version: 1,
                credential_id: Some(credential_id),
                config: serde_json::json!({ "method": "GET", "url": "https://example.com/" })
                    .as_object()
                    .cloned()
                    .unwrap(),
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }],
            edges: vec![Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            }],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Manual,
                name: None,
                description: None,
            }],
        }
    }

    fn odoo_data() -> serde_json::Value {
        serde_json::json!({
            "base_url": "https://odoo.example.com",
            "database": "prod",
            "username": "bot",
            "api_key": "secret-key",
        })
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_supplied_credential_validates_and_seals_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Production Odoo".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(odoo_data()),
            })
            .await
            .unwrap();

        assert_eq!(created.org_id, org_id);
        assert_eq!(created.kind, "odoo_api");
        assert_eq!(created.name, "Production Odoo");
        assert_eq!(created.origin, CredentialOrigin::Supplied);

        let found = usecase.find_credential(org_id, created.id).await.unwrap();
        assert_eq!(found, Some(created));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_supplied_credential_reveals_no_secret_and_keeps_readable_values_apart() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create-echo").await;
        let usecase = use_case(pool.clone());

        let (created, revealed) = usecase
            .create_credential(CreateCredentialCommand {
                description: Some("Instance de production".to_owned()),
                expires_at: None,
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Production Odoo".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(odoo_data()),
            })
            .await
            .unwrap();

        assert_eq!(revealed, None);
        assert_eq!(created.description, "Instance de production");
        assert_eq!(
            created.data,
            odoo_data()
                .as_object()
                .map(|object| object
                    .iter()
                    .filter(|(name, _)| name.as_str() != "api_key")
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect::<serde_json::Map<_, _>>())
                .unwrap()
        );
        assert_eq!(created.secrets_set, vec!["api_key".to_owned()]);
        assert_eq!(created.status, CredentialStatus::Valid);

        let row = sqlx::query!(
            "SELECT data, data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            created.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(row.data.get("api_key").is_none());
        let cipher = SecretCipher::new(&[7u8; 32]).unwrap();
        let sealed = cipher
            .open(&SealedSecret {
                nonce: row.data_nonce,
                ciphertext: row.data_ciphertext,
            })
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&sealed).unwrap(),
            serde_json::json!({ "api_key": "secret-key" })
        );
    }

    /// The other half of the same acceptance criterion: for a `Generated`
    /// credential, the plaintext returned at creation is the only time
    /// Mestier's own freshly minted secret is ever visible outside the
    /// sealed column — proven here by checking it against what the
    /// worker-only `open_credential` later reads back from storage.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_generated_credential_returns_the_secret_that_was_actually_sealed() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create-generated-secret").await;
        let usecase = use_case(pool);

        let (created, secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Outgoing signature".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();

        let (_, opened) = usecase.open_credential(org_id, created.id).await.unwrap();
        assert_eq!(secret, Some(opened));
        assert_eq!(created.secrets_set, vec!["token".to_owned()]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_credential_with_a_missing_field_is_refused_and_names_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "invalid").await;
        let usecase = use_case(pool);

        let mut data = odoo_data();
        data.as_object_mut().unwrap().remove("api_key");

        let error = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Broken".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(data),
            })
            .await
            .expect_err("a credential missing its key cannot authenticate");

        assert!(format!("{error}").contains("Clé d’API"), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_credential_with_an_unknown_kind_is_refused_and_names_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "unknown-kind").await;
        let usecase = use_case(pool);

        let error = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "not_a_real_kind".to_owned(),
                name: "Broken".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({})),
            })
            .await
            .expect_err("an unknown scheme has nothing to validate against");

        assert!(format!("{error}").contains("not_a_real_kind"), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_credential_without_a_configured_cipher_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "no-cipher").await;
        let usecase = use_case_without_cipher(pool);

        let error = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .expect_err("an instance with no automation secret key cannot seal anything");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_only_returns_the_requesting_organizations_credentials() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list").await;
        let other_org = seed_organization(&pool, "list-other").await;
        let usecase = use_case(pool);

        usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Mine".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id: other_org,
                kind: "bearer_token".to_owned(),
                name: "Not mine".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let listed = usecase.list_credentials(org_id).await.unwrap();

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Mine");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn finding_a_credential_from_another_organization_reads_back_absent() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "find-owner").await;
        let stranger_org = seed_organization(&pool, "find-stranger").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let found = usecase
            .find_credential(stranger_org, created.id)
            .await
            .unwrap();

        assert_eq!(found, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_can_rename_without_resupplying_data() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "update-rename").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Old name".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let updated = usecase
            .update_credential(UpdateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                id: created.id,
                name: Some("New name".to_owned()),
                data: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "New name");
        assert_eq!(updated.kind, "bearer_token");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_missing_credential_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "update-missing").await;
        let usecase = use_case(pool);

        let error = usecase
            .update_credential(UpdateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                id: generate_uuid_v7(),
                name: Some("Nope".to_owned()),
                data: None,
            })
            .await
            .expect_err("there is nothing to update");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_credential_removes_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        usecase.delete_credential(org_id, created.id).await.unwrap();

        assert_eq!(
            usecase.find_credential(org_id, created.id).await.unwrap(),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_used_credential_succeeds_and_remembers_its_name() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-referenced").await;
        let usecase = use_case(pool.clone());

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Jeton fournisseur".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Uses the token".to_owned(),
                description: None,
            })
            .await
            .unwrap();
        let graph = graph_using(created.id);
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph: graph.clone(),
                    layout: None,
                    created_by: None,
                },
            )
            .await
            .unwrap();

        usecase.delete_credential(org_id, created.id).await.unwrap();

        assert_eq!(
            usecase.find_credential(org_id, created.id).await.unwrap(),
            None
        );
        assert_eq!(
            usecase.missing_credential_message(org_id, created.id).await,
            "credential `Jeton fournisseur` no longer exists"
        );
        let unknown = generate_uuid_v7();
        assert_eq!(
            usecase.missing_credential_message(org_id, unknown).await,
            format!("credential `{unknown}` no longer exists")
        );

        let saved_again = usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph,
                    layout: None,
                    created_by: None,
                },
            )
            .await;
        assert!(
            saved_again.is_err(),
            "a graph naming a credential that no longer exists is still refused on save"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_deleted_credentials_name_is_not_visible_to_another_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "tombstone-owner").await;
        let stranger_org = seed_organization(&pool, "tombstone-stranger").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Secret name".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        usecase.delete_credential(org_id, created.id).await.unwrap();

        let message = usecase
            .missing_credential_message(stranger_org, created.id)
            .await;

        assert!(!message.contains("Secret name"), "{message}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_an_unreferenced_credential_still_succeeds() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-unreferenced").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Does not use it".to_owned(),
                description: None,
            })
            .await
            .unwrap();

        usecase.delete_credential(org_id, created.id).await.unwrap();

        assert_eq!(
            usecase.find_credential(org_id, created.id).await.unwrap(),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_credential_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-owner").await;
        let stranger_org = seed_organization(&pool, "delete-stranger").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let error = usecase
            .delete_credential(stranger_org, created.id)
            .await
            .expect_err("a stranger's delete must not succeed");
        assert!(matches!(error, CoreError::NotFound));

        assert!(
            usecase
                .find_credential(org_id, created.id)
                .await
                .unwrap()
                .is_some(),
            "the credential must still be there"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn rotating_a_generated_credential_produces_new_sealed_bytes() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "rotate-generated").await;
        let usecase = use_case(pool.clone());

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Outgoing signature".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();
        assert_eq!(created.origin, CredentialOrigin::Generated);

        let before = sqlx::query!(
            "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            created.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let (rotated, _secret) = usecase.rotate_credential(org_id, created.id).await.unwrap();
        assert_eq!(rotated.id, created.id);

        let after = sqlx::query!(
            "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            created.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_ne!(before.data_nonce, after.data_nonce);
        assert_ne!(before.data_ciphertext, after.data_ciphertext);
    }

    /// The acceptance criterion (#203) on the rotation half: the returned
    /// secret is the fresh one actually sealed, different from what creation
    /// returned, and matches what `open_credential` reads back afterwards.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn rotating_a_generated_credential_returns_the_freshly_sealed_secret() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "rotate-returns-secret").await;
        let usecase = use_case(pool.clone());

        let (created, created_secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Outgoing signature".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();

        let (_, rotated_secret) = usecase.rotate_credential(org_id, created.id).await.unwrap();

        assert_ne!(
            Some(rotated_secret.clone()),
            created_secret,
            "a rotation must mint a genuinely new secret"
        );
        let (_, opened) = usecase.open_credential(org_id, created.id).await.unwrap();
        assert_eq!(rotated_secret, opened);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn rotating_a_supplied_credential_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "rotate-supplied").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Supplied token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let error = usecase
            .rotate_credential(org_id, created.id)
            .await
            .expect_err("a supplied credential has nothing of Mestier's own to regenerate");

        assert!(format!("{error}").contains("ne se régénère pas"), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn opening_a_supplied_credential_returns_its_validated_fields() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "open-supplied").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Production Odoo".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(odoo_data()),
            })
            .await
            .unwrap();

        let (opened, plaintext) = usecase.open_credential(org_id, created.id).await.unwrap();

        assert_eq!(opened.id, created.id);
        let fields: serde_json::Value = serde_json::from_slice(&plaintext).unwrap();
        assert_eq!(fields, odoo_data());
    }

    /// A `Generated` credential's plaintext is the raw signing secret, not
    /// JSON — the same bytes `create_credential` sealed, byte for byte.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn opening_a_generated_credential_returns_its_raw_secret_bytes() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "open-generated").await;
        let usecase = use_case(pool.clone());

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Outgoing signature".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();

        let (_, plaintext) = usecase.open_credential(org_id, created.id).await.unwrap();

        // Never valid JSON by construction (32 random bytes), which is
        // exactly the point: a `Generated` credential's bytes are used
        // as-is, not decoded.
        assert_eq!(plaintext.len(), 32);
        assert!(serde_json::from_slice::<serde_json::Value>(&plaintext).is_err());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn opening_a_credential_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "open-owner").await;
        let stranger_org = seed_organization(&pool, "open-stranger").await;
        let usecase = use_case(pool);

        let (created, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let error = usecase
            .open_credential(stranger_org, created.id)
            .await
            .expect_err("a stranger must not open another organization's credential");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn opening_an_unknown_credential_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "open-missing").await;
        let usecase = use_case(pool);

        let error = usecase
            .open_credential(org_id, generate_uuid_v7())
            .await
            .expect_err("there is nothing to open");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn opening_a_credential_without_a_configured_cipher_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "open-no-cipher").await;
        let usecase_with_cipher = use_case(pool.clone());
        let (created, _secret) = usecase_with_cipher
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let usecase_without_cipher = use_case_without_cipher(pool);
        let error = usecase_without_cipher
            .open_credential(org_id, created.id)
            .await
            .expect_err("an instance with no automation secret key cannot open anything");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    async fn create_ssh(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        name: &str,
    ) -> Credential {
        let (created, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "ssh".to_owned(),
                name: name.to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({
                    "host": "srv.example.com", "port": 22, "username": "root",
                    "method": "private-key", "private_key": "KEY", "passphrase": "PHRASE",
                })),
            })
            .await
            .unwrap();
        created
    }

    async fn opened_fields(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        id: Uuid,
    ) -> serde_json::Value {
        let (_, plaintext) = usecase.open_credential(org_id, id).await.unwrap();
        serde_json::from_slice(&plaintext).unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn changing_one_secret_keeps_the_others_and_the_readable_values() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "partial").await;
        let usecase = use_case(pool);
        let created = create_ssh(&usecase, org_id, "Serveur").await;

        let updated = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: created.id,
                name: None,
                description: None,
                data: Some(serde_json::json!({ "passphrase": "NEW" })),
                expires_at: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.secrets_set, vec!["passphrase", "private_key"]);
        assert_eq!(updated.data["host"], serde_json::json!("srv.example.com"));
        let fields = opened_fields(&usecase, org_id, created.id).await;
        assert_eq!(fields["private_key"], serde_json::json!("KEY"));
        assert_eq!(fields["passphrase"], serde_json::json!("NEW"));
        assert_eq!(fields["host"], serde_json::json!("srv.example.com"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_null_secret_in_an_update_is_refused_and_leaves_the_credential_untouched() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "null-secret").await;
        let usecase = use_case(pool);
        let created = create_ssh(&usecase, org_id, "Serveur").await;

        let error = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: created.id,
                name: Some("Renamed".to_owned()),
                description: None,
                data: Some(serde_json::json!({ "passphrase": null })),
                expires_at: None,
            })
            .await
            .expect_err("an explicit null is refused");

        let CoreError::Conflict(message) = &error else {
            panic!("a validation failure travels as a conflict: {error}");
        };
        assert!(
            CredentialError::validation_message(message).is_some(),
            "{message}"
        );
        let after = usecase
            .find_credential(org_id, created.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.name, "Serveur");
        assert_eq!(
            opened_fields(&usecase, org_id, created.id).await["passphrase"],
            "PHRASE"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_description_and_an_expiry_are_stored_and_the_expiry_makes_it_expired() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "expiry").await;
        let usecase = use_case(pool);
        let (created, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: Some("  À renouveler  ".to_owned()),
                expires_at: Some(Utc::now() - chrono::Duration::days(1)),
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Jeton".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();

        let listed = usecase.list_credentials(org_id).await.unwrap();

        assert_eq!(listed[0].description, "À renouveler");
        assert_eq!(listed[0].status_at(Utc::now()), CredentialStatus::Expired);

        let renewed = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: created.id,
                name: None,
                description: None,
                data: None,
                expires_at: Some(Some(Utc::now() + chrono::Duration::days(30))),
            })
            .await
            .unwrap();
        assert_eq!(renewed.status_at(Utc::now()), CredentialStatus::Valid);

        let cleared = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: created.id,
                name: None,
                description: None,
                data: None,
                expires_at: Some(None),
            })
            .await
            .unwrap();
        assert_eq!(cleared.expires_at, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_description_longer_than_five_hundred_characters_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "long-description").await;
        let usecase = use_case(pool);

        let error = usecase
            .create_credential(CreateCredentialCommand {
                description: Some("x".repeat(501)),
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Jeton".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .expect_err("501 characters is one too many");

        assert!(format!("{error}").contains("500 caractères"), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_same_name_twice_for_a_kind_answers_a_static_french_conflict() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "same-name").await;
        let usecase = use_case(pool);
        create_ssh(&usecase, org_id, "Serveur").await;

        let error = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "ssh".to_owned(),
                name: "Serveur".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({
                    "host": "h", "port": 22, "username": "u", "private_key": "K",
                })),
            })
            .await
            .expect_err("the name is taken");

        assert!(
            matches!(&error, CoreError::Conflict(message) if message == "Une identification de ce type porte déjà ce nom."),
            "{error}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn renaming_onto_a_taken_name_is_the_same_conflict() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "rename-onto").await;
        let usecase = use_case(pool);
        create_ssh(&usecase, org_id, "Premier").await;
        let second = create_ssh(&usecase, org_id, "Second").await;

        let error = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: second.id,
                name: Some("Premier".to_owned()),
                description: None,
                data: None,
                expires_at: None,
            })
            .await
            .expect_err("the name is taken");

        assert!(
            matches!(&error, CoreError::Conflict(message) if message == "Une identification de ce type porte déjà ce nom."),
            "{error}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_generated_credential_cannot_be_patched_field_by_field() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "generated-patch").await;
        let usecase = use_case(pool);
        let (created, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Signature".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();

        let error = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: created.id,
                name: None,
                description: None,
                data: Some(serde_json::json!({ "token": "forged" })),
                expires_at: None,
            })
            .await
            .expect_err("a generated secret is regenerated, not typed");

        assert!(format!("{error}").contains("régénérez-la"), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_a_secret_on_a_credential_that_failed_to_refresh_makes_it_valid_again() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "revive").await;
        let usecase = use_case(pool.clone());
        let (created, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: Some(Utc::now() - chrono::Duration::days(2)),
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Jeton".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "old" })),
            })
            .await
            .unwrap();
        sqlx::query!(
            "UPDATE automation.credential SET status = 'expired' WHERE id = $1",
            created.id
        )
        .execute(&pool)
        .await
        .unwrap();

        let revived = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: created.id,
                name: None,
                description: None,
                data: Some(serde_json::json!({ "token": "new" })),
                expires_at: None,
            })
            .await
            .unwrap();

        assert_eq!(revived.status, CredentialStatus::Valid);
        assert_eq!(revived.expires_at, None);
        assert_eq!(revived.status_at(Utc::now()), CredentialStatus::Valid);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_written_before_the_split_is_split_on_first_read_and_still_opens() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "legacy").await;
        let usecase = use_case(pool.clone());
        let cipher = SecretCipher::new(&[7u8; 32]).unwrap();
        let sealed = cipher.seal(odoo_data().to_string().as_bytes()).unwrap();
        let id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO automation.credential
                   (id, org_id, kind, name, origin, data_nonce, data_ciphertext, secret_names)
               VALUES ($1, $2, 'odoo_api', 'Legacy Odoo', 'supplied', $3, $4, ARRAY['api_key'])"#,
            id,
            org_id.0,
            sealed.nonce,
            sealed.ciphertext,
        )
        .execute(&pool)
        .await
        .unwrap();

        let (_, before) = usecase.open_credential(org_id, id).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&before).unwrap(),
            odoo_data()
        );

        let listed = usecase.list_credentials(org_id).await.unwrap();

        assert_eq!(listed[0].data["database"], serde_json::json!("prod"));
        assert!(!listed[0].data.contains_key("api_key"));
        assert_eq!(listed[0].secrets_set, vec!["api_key".to_owned()]);
        assert_eq!(listed[0].status, CredentialStatus::Valid);
        assert!(listed[0].fields_split);
        let (_, after) = usecase.open_credential(org_id, id).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&after).unwrap(),
            odoo_data()
        );
        let row = sqlx::query!(
            "SELECT data, updated_at, created_at FROM automation.credential WHERE id = $1",
            id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(row.data.get("api_key").is_none());
        assert_eq!(row.updated_at, row.created_at, "the split is not an edit");
    }

    async fn use_in_workflow(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        credential_id: Uuid,
        workflow_name: &str,
    ) -> Uuid {
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: workflow_name.to_owned(),
                description: None,
            })
            .await
            .unwrap();
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph: graph_using(credential_id),
                    layout: None,
                    created_by: None,
                },
            )
            .await
            .unwrap();
        workflow.id
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_usage_counts_workflows_whose_current_version_uses_the_credential() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "usage").await;
        let other_org = seed_organization(&pool, "usage-other").await;
        let usecase = use_case(pool);
        let make = |org: OrganizationId, name: &'static str| {
            let usecase = usecase.clone();
            async move {
                usecase
                    .create_credential(CreateCredentialCommand {
                        description: None,
                        expires_at: None,
                        org_id: org,
                        kind: "bearer_token".to_owned(),
                        name: name.to_owned(),
                        origin: CredentialOrigin::Supplied,
                        data: Some(serde_json::json!({ "token": "abc" })),
                    })
                    .await
                    .unwrap()
                    .0
            }
        };
        let used = make(org_id, "Utilisée").await;
        let unused = make(org_id, "Inutilisée").await;
        let foreign = make(other_org, "Autre organisation").await;
        use_in_workflow(&usecase, org_id, used.id, "Premier").await;
        let second = use_in_workflow(&usecase, org_id, used.id, "Second").await;
        use_in_workflow(&usecase, other_org, foreign.id, "Chez l’autre").await;

        let usage = usecase.credential_usage(org_id).await.unwrap();

        assert_eq!(usage.get(&used.id), Some(&2));
        assert_eq!(usage.get(&unused.id), Some(&0));
        assert_eq!(
            usage.get(&foreign.id),
            None,
            "another organization's credential is not listed"
        );

        let mut without = graph_using(used.id);
        without.connectors[0].credential_id = None;
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: second,
                    graph: without,
                    layout: None,
                    created_by: None,
                },
            )
            .await
            .unwrap();

        let usage = usecase.credential_usage(org_id).await.unwrap();
        assert_eq!(
            usage.get(&used.id),
            Some(&1),
            "an old version that used it no longer counts"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_usage_counts_a_webhook_trigger_and_an_outgoing_signing_key() {
        use crate::application::automation::trigger_fixtures::{graph_of, webhook_with};
        use crate::domain::automation::workflow::OnInvalid;

        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "usage-kinds").await;
        let usecase = use_case(pool);
        let (credential, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Entrant".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        let (signing, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Signature".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();
        let hooked = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Webhook".to_owned(),
                description: None,
            })
            .await
            .unwrap();
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: hooked.id,
                    graph: graph_of(vec![(
                        "t1",
                        webhook_with(
                            &format!("usage-{}", Uuid::now_v7().simple()),
                            Some(credential.id),
                            Vec::new(),
                            OnInvalid::Reject,
                        ),
                    )]),
                    layout: None,
                    created_by: None,
                },
            )
            .await
            .unwrap();
        let signed = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Signé".to_owned(),
                description: None,
            })
            .await
            .unwrap();
        let mut graph = graph_using(credential.id);
        graph.connectors[0].credential_id = None;
        graph.connectors[0].config.insert(
            "signing_credential_id".to_owned(),
            serde_json::json!(signing.id),
        );
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: signed.id,
                    graph,
                    layout: None,
                    created_by: None,
                },
            )
            .await
            .unwrap();

        let usage = usecase.credential_usage(org_id).await.unwrap();

        assert_eq!(usage.get(&credential.id), Some(&1));
        assert_eq!(usage.get(&signing.id), Some(&1));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_engine_fails_the_node_naming_a_credential_deleted_after_the_workflow_was_saved() {
        use crate::infrastructure::automation::connectors::ConnectorRegistry;
        use crate::infrastructure::automation::worker::WorkerSchedule;

        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "run-deleted").await;
        let usecase = use_case(pool.clone());
        let (credential, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Jeton fournisseur".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        let workflow_id = use_in_workflow(&usecase, org_id, credential.id, "Appelle l’API").await;
        usecase
            .delete_credential(org_id, credential.id)
            .await
            .unwrap();
        let run_id = usecase
            .start_run(org_id, workflow_id, serde_json::json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", WorkerSchedule::default())
            .await
            .unwrap();

        let error: Option<String> = sqlx::query_scalar!(
            "SELECT error FROM automation.run_step WHERE run_id = $1 AND connector_id = 'c1'",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            error.as_deref(),
            Some("credential `Jeton fournisseur` no longer exists")
        );
    }

    static MIGRATION_SCRATCH: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_credential_migration_backfills_existing_rows_reverts_and_applies_again() {
        use crate::application::test_support::scratch_pool;

        const PREVIOUS: i64 = 20261106000002;

        let pool = scratch_pool("mestier_credmig_test_", &MIGRATION_SCRATCH).await;
        let migrator = sqlx::migrate::Migrator::new(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
        )
        .await
        .unwrap();
        migrator.undo(&pool, PREVIOUS).await.unwrap();

        let columns: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM information_schema.columns
             WHERE table_schema = 'automation' AND table_name = 'credential' AND column_name = 'description'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(columns, 0, "the revert removes the new columns");
        let tombstones: Option<String> =
            sqlx::query_scalar("SELECT to_regclass('automation.credential_tombstone')::text")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(tombstones, None, "the revert removes the tombstone table");

        let org_id = seed_organization(&pool, "migration").await;
        let seal = |text: &str| {
            SecretCipher::new(&[7u8; 32])
                .unwrap()
                .seal(text.as_bytes())
                .unwrap()
        };
        let mut ids = Vec::new();
        for (kind, name, origin) in [
            ("http_header", "Supplied header", "supplied"),
            ("http_basic", "Supplied basic", "supplied"),
            ("bearer_token", "Supplied bearer", "supplied"),
            ("odoo_api", "Supplied odoo", "supplied"),
            ("bearer_token", "Generated bearer", "generated"),
        ] {
            let sealed = seal("{}");
            let id = generate_uuid_v7();
            sqlx::query(
                "INSERT INTO automation.credential
                     (id, org_id, kind, name, origin, data_nonce, data_ciphertext)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(org_id.0)
            .bind(kind)
            .bind(name)
            .bind(origin)
            .bind(sealed.nonce)
            .bind(sealed.ciphertext)
            .execute(&pool)
            .await
            .unwrap();
            ids.push(id);
        }

        migrator.run(&pool).await.unwrap();

        let rows: Vec<(String, String, String, Vec<String>, bool, serde_json::Value)> =
            sqlx::query_as(
                "SELECT name, status, description, secret_names, fields_split, data
                 FROM automation.credential WHERE org_id = $1 ORDER BY name",
            )
            .bind(org_id.0)
            .fetch_all(&pool)
            .await
            .unwrap();
        let by_name = |wanted: &str| rows.iter().find(|row| row.0 == wanted).unwrap();
        assert!(rows.iter().all(|row| row.1 == "valid" && row.2.is_empty()));
        assert!(rows.iter().all(|row| row.5 == serde_json::json!({})));
        assert_eq!(
            by_name("Supplied header").3,
            vec!["header_value".to_owned()]
        );
        assert_eq!(by_name("Supplied basic").3, vec!["password".to_owned()]);
        assert_eq!(by_name("Supplied bearer").3, vec!["token".to_owned()]);
        assert_eq!(by_name("Supplied odoo").3, vec!["api_key".to_owned()]);
        assert_eq!(by_name("Generated bearer").3, vec!["token".to_owned()]);
        assert!(
            !by_name("Supplied header").4,
            "a supplied row is split on first read"
        );
        assert!(
            by_name("Generated bearer").4,
            "a generated row has nothing to split"
        );

        migrator.undo(&pool, PREVIOUS).await.unwrap();
        let surviving: i64 =
            sqlx::query_scalar("SELECT count(*) FROM automation.credential WHERE org_id = $1")
                .bind(org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(surviving, 5, "the revert keeps the rows");

        migrator.run(&pool).await.unwrap();
        let again: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM automation.credential WHERE org_id = $1 AND status = 'valid'",
        )
        .bind(org_id.0)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(again, 5);
    }

    async fn sealed_object(
        pool: &PgPool,
        usecase: &MestierUseCase,
        id: Uuid,
    ) -> serde_json::Map<String, serde_json::Value> {
        let row = sqlx::query!(
            "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await
        .unwrap();
        open_object(
            usecase.cipher.as_ref().unwrap(),
            &SealedSecret {
                nonce: row.data_nonce,
                ciphertext: row.data_ciphertext,
            },
        )
        .unwrap()
    }

    fn bearer(org_id: OrganizationId, name: &str) -> CreateCredentialCommand {
        CreateCredentialCommand {
            description: None,
            expires_at: None,
            org_id,
            kind: "bearer_token".to_owned(),
            name: name.to_owned(),
            origin: CredentialOrigin::Supplied,
            data: Some(serde_json::json!({ "token": "abc" })),
        }
    }

    fn validation_message(error: CoreError) -> String {
        let CoreError::Conflict(message) = error else {
            panic!("expected a validation conflict, got {error:?}");
        };
        CredentialError::validation_message(&message)
            .unwrap_or_else(|| panic!("not a validation message: {message}"))
            .to_owned()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn reverting_the_credential_migration_is_refused_while_a_supplied_credential_holds_split_fields()
     {
        use crate::application::test_support::scratch_pool;

        const DOWN: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/20261107000001_complete_automation_credentials.down.sql"
        ));
        const UP: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/20261107000001_complete_automation_credentials.up.sql"
        ));
        let pool = scratch_pool("mestier_credmig_test_", &MIGRATION_SCRATCH).await;
        let has_column = || async {
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM information_schema.columns
                 WHERE table_schema = 'automation' AND table_name = 'credential'
                   AND column_name = 'fields_split'",
            )
            .fetch_one(&pool)
            .await
            .unwrap()
        };
        sqlx::query("DELETE FROM automation.credential WHERE fields_split AND origin = 'supplied'")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::raw_sql(DOWN).execute(&pool).await.unwrap();
        assert_eq!(
            has_column().await,
            0,
            "nothing is split: the revert goes through"
        );
        sqlx::raw_sql(UP).execute(&pool).await.unwrap();
        assert_eq!(has_column().await, 1);

        let org_id = seed_organization(&pool, "migration-guard").await;
        let sealed = SecretCipher::new(&[7u8; 32]).unwrap().seal(b"{}").unwrap();
        let id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO automation.credential
                 (id, org_id, kind, name, origin, data_nonce, data_ciphertext, fields_split)
             VALUES ($1, $2, 'bearer_token', 'Split', 'supplied', $3, $4, TRUE)",
        )
        .bind(id)
        .bind(org_id.0)
        .bind(sealed.nonce)
        .bind(sealed.ciphertext)
        .execute(&pool)
        .await
        .unwrap();

        let refusal = sqlx::raw_sql(DOWN).execute(&pool).await.unwrap_err();

        let message = refusal.to_string();
        assert!(
            message.contains("cannot revert 20261107000001")
                && message.contains("1 supplied credential"),
            "{message}"
        );
        assert_eq!(has_column().await, 1, "a refused revert changes nothing");
        sqlx::query("DELETE FROM automation.credential WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_organization_holds_at_most_two_hundred_credentials_even_when_created_together() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "quota").await;
        let usecase = use_case(pool.clone());
        sqlx::query(
            "INSERT INTO automation.credential
                 (id, org_id, kind, name, origin, data_nonce, data_ciphertext, fields_split)
             SELECT gen_random_uuid(), $1, 'bearer_token', 'Fill ' || n, 'supplied',
                    '\\x00'::bytea, '\\x00'::bytea, TRUE
             FROM generate_series(1, 195) AS n",
        )
        .bind(org_id.0)
        .execute(&pool)
        .await
        .unwrap();

        let mut creations = tokio::task::JoinSet::new();
        for index in 0..12 {
            let usecase = usecase.clone();
            creations.spawn(async move {
                usecase
                    .create_credential(bearer(org_id, &format!("Late {index}")))
                    .await
            });
        }
        let mut created = 0;
        let mut refused = Vec::new();
        while let Some(joined) = creations.join_next().await {
            match joined.unwrap() {
                Ok(_) => created += 1,
                Err(error) => refused.push(error),
            }
        }

        assert_eq!(created, 5, "195 + 5 = 200, whatever the interleaving");
        assert_eq!(refused.len(), 7);
        for error in refused {
            let CoreError::Conflict(message) = error else {
                panic!("expected a conflict, got {error:?}");
            };
            assert!(
                CredentialError::validation_message(&message).is_none(),
                "a quota is a plain conflict, not a field error: {message}"
            );
            assert!(!message.contains("200 "), "{message}");
        }
        let total: i64 =
            sqlx::query_scalar("SELECT count(*) FROM automation.credential WHERE org_id = $1")
                .bind(org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(total, 200);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_kind_that_cannot_be_generated_refuses_the_generated_origin() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "generated").await;
        let usecase = use_case(pool);

        for kind in [
            "hmac",
            "oauth1",
            "oauth2_authorization_code",
            "http_basic",
            "odoo_api",
        ] {
            let error = usecase
                .create_credential(CreateCredentialCommand {
                    description: None,
                    expires_at: None,
                    org_id,
                    kind: kind.to_owned(),
                    name: format!("Generated {kind}"),
                    origin: CredentialOrigin::Generated,
                    data: None,
                })
                .await
                .unwrap_err();

            assert!(
                validation_message(error).contains("ne peut pas être généré"),
                "{kind}"
            );
        }
        let (bearer, revealed) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Generated bearer".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();
        assert!(revealed.is_some());
        assert_eq!(bearer.origin, CredentialOrigin::Generated);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_name_over_a_hundred_and_twenty_characters_or_with_a_nul_is_refused_on_create_and_update()
     {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "bounds").await;
        let usecase = use_case(pool);
        let (credential, _) = usecase
            .create_credential(bearer(org_id, "Normal"))
            .await
            .unwrap();

        let too_long = usecase
            .create_credential(bearer(org_id, &"n".repeat(121)))
            .await
            .unwrap_err();
        let with_nul = usecase
            .create_credential(bearer(org_id, "a\0b"))
            .await
            .unwrap_err();
        let renamed = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: credential.id,
                name: Some("n".repeat(121)),
                description: None,
                data: None,
                expires_at: None,
            })
            .await
            .unwrap_err();
        let secret_with_nul = usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: credential.id,
                name: None,
                description: None,
                data: Some(serde_json::json!({ "token": "a\0b" })),
                expires_at: None,
            })
            .await
            .unwrap_err();

        assert!(validation_message(too_long).contains("120"));
        assert!(validation_message(with_nul).contains("caractère interdit"));
        assert!(validation_message(renamed).contains("120"));
        assert!(validation_message(secret_with_nul).contains("caractère interdit"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn provider_urls_are_https_on_a_closed_instance_and_plaintext_never_travels_over_http() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "https").await;
        let closed = use_case(pool.clone());
        let open = use_case(pool).with_private_network(PrivateNetworkAccess::Allowed);
        let oauth = |name: &str, url: &str| CreateCredentialCommand {
            description: None,
            expires_at: None,
            org_id,
            kind: "oauth2_client_credentials".to_owned(),
            name: name.to_owned(),
            origin: CredentialOrigin::Supplied,
            data: Some(serde_json::json!({
                "token_url": url,
                "client_id": "id",
                "client_secret": "secret",
            })),
        };
        let one = |name: &str, scheme: &str, method: &str| CreateCredentialCommand {
            description: None,
            expires_at: None,
            org_id,
            kind: "oauth1".to_owned(),
            name: name.to_owned(),
            origin: CredentialOrigin::Supplied,
            data: Some(serde_json::json!({
                "consumer_key": "ck",
                "consumer_secret": "cs",
                "request_token_url": format!("{scheme}://a.example.com/request"),
                "authorization_url": "https://a.example.com/authorize",
                "access_token_url": format!("{scheme}://a.example.com/access"),
                "signature_method": method,
            })),
        };

        let refused = closed
            .create_credential(oauth("Http closed", "http://auth.example.com/token"))
            .await
            .unwrap_err();
        assert!(validation_message(refused).contains("https"));
        closed
            .create_credential(oauth("Https closed", "https://auth.example.com/token"))
            .await
            .unwrap();
        open.create_credential(oauth("Http open", "http://auth.example.com/token"))
            .await
            .unwrap();
        open.create_credential(one("Hmac over http", "http", "HMAC-SHA1"))
            .await
            .unwrap();

        let plaintext = open
            .create_credential(one("Plaintext over http", "http", "PLAINTEXT"))
            .await
            .unwrap_err();
        assert!(validation_message(plaintext).contains("PLAINTEXT"));
        open.create_credential(one("Plaintext over https", "https", "PLAINTEXT"))
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_survives_a_generated_secret_and_a_row_nobody_can_open_and_still_splits_the_others()
     {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "legacy").await;
        let usecase = use_case(pool.clone());
        let (generated, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Generated".to_owned(),
                origin: CredentialOrigin::Generated,
                data: None,
            })
            .await
            .unwrap();
        let (legacy, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Legacy".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({
                    "base_url": "https://odoo.example.com",
                    "database": "prod",
                    "username": "bot",
                    "api_key": "KEY-VALUE",
                })),
            })
            .await
            .unwrap();
        let foreign_key = SecretCipher::new(&[9u8; 32]).unwrap();
        let unreadable = foreign_key
            .seal(br#"{"token":"LOST-SECRET-VALUE"}"#)
            .unwrap();
        let (stranger, _) = usecase
            .create_credential(bearer(org_id, "Sealed elsewhere"))
            .await
            .unwrap();
        sqlx::query!(
            "UPDATE automation.credential SET fields_split = FALSE, data = '{}'::jsonb WHERE id = ANY($1)",
            &[generated.id, legacy.id, stranger.id]
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            "UPDATE automation.credential SET data_nonce = $2, data_ciphertext = $3 WHERE id = $1",
            stranger.id,
            unreadable.nonce,
            unreadable.ciphertext
        )
        .execute(&pool)
        .await
        .unwrap();
        let everything = seal_object(
            usecase.cipher.as_ref().unwrap(),
            serde_json::json!({
                "base_url": "https://odoo.example.com",
                "database": "prod",
                "username": "bot",
                "api_key": "KEY-VALUE",
            })
            .as_object()
            .unwrap(),
        )
        .unwrap();
        sqlx::query!(
            "UPDATE automation.credential SET data_nonce = $2, data_ciphertext = $3 WHERE id = $1",
            legacy.id,
            everything.nonce,
            everything.ciphertext
        )
        .execute(&pool)
        .await
        .unwrap();

        let listed = usecase.list_credentials(org_id).await.unwrap();

        assert_eq!(listed.len(), 3, "no credential takes the others down");
        let legacy_after = listed.iter().find(|c| c.id == legacy.id).unwrap();
        assert!(legacy_after.fields_split, "a readable legacy row is split");
        assert_eq!(
            legacy_after.data.get("base_url"),
            Some(&serde_json::json!("https://odoo.example.com"))
        );
        let flags: Vec<(Uuid, bool)> =
            sqlx::query_as("SELECT id, fields_split FROM automation.credential WHERE id = ANY($1)")
                .bind(vec![generated.id, stranger.id])
                .fetch_all(&pool)
                .await
                .unwrap();
        assert!(
            flags.iter().all(|(_, split)| !split),
            "a generated secret and an unreadable row are left as they are"
        );
        assert!(
            usecase
                .find_credential(org_id, stranger.id)
                .await
                .unwrap()
                .is_some()
        );
    }

    async fn hold_the_row(pool: &PgPool, id: Uuid) -> sqlx::Transaction<'static, sqlx::Postgres> {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT 1 FROM automation.credential WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        tx
    }

    fn patch(org_id: OrganizationId, id: Uuid, data: serde_json::Value) -> UpdateCredentialCommand {
        UpdateCredentialCommand {
            org_id,
            id,
            name: None,
            description: None,
            data: Some(data),
            expires_at: None,
        }
    }

    fn mtls(org_id: OrganizationId) -> CreateCredentialCommand {
        CreateCredentialCommand {
            description: None,
            expires_at: None,
            org_id,
            kind: "mtls".to_owned(),
            name: "Client".to_owned(),
            origin: CredentialOrigin::Supplied,
            data: Some(serde_json::json!({
                "certificate": "-----BEGIN CERTIFICATE-----",
                "private_key": "KEY-ONE",
            })),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_patches_on_two_secrets_keep_both_whatever_the_interleaving() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "patch-both").await;
        let usecase = use_case(pool.clone());
        let (credential, _) = usecase.create_credential(mtls(org_id)).await.unwrap();

        let holder = hold_the_row(&pool, credential.id).await;
        let first = tokio::spawn({
            let usecase = usecase.clone();
            async move {
                usecase
                    .update_credential(patch(
                        org_id,
                        credential.id,
                        serde_json::json!({ "passphrase": "PASS-TWO" }),
                    ))
                    .await
            }
        });
        let second = tokio::spawn({
            let usecase = usecase.clone();
            async move {
                usecase
                    .update_credential(patch(
                        org_id,
                        credential.id,
                        serde_json::json!({ "private_key": "KEY-THREE" }),
                    ))
                    .await
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        holder.commit().await.unwrap();
        first.await.unwrap().unwrap();
        second.await.unwrap().unwrap();

        let sealed = sealed_object(&pool, &usecase, credential.id).await;
        assert_eq!(sealed["passphrase"], serde_json::json!("PASS-TWO"));
        assert_eq!(sealed["private_key"], serde_json::json!("KEY-THREE"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_patch_waiting_behind_a_token_refresh_keeps_the_fresh_tokens_instead_of_rewriting_old_ones()
     {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "patch-refresh").await;
        let usecase = use_case(pool.clone());
        let (credential, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "oauth2_authorization_code".to_owned(),
                name: "Compte".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({
                    "authorization_url": "https://auth.example.com/authorize",
                    "token_url": "https://auth.example.com/token",
                    "client_id": "id",
                    "client_secret": "secret",
                    "scopes": "openid",
                })),
            })
            .await
            .unwrap();
        let cipher = usecase.cipher.clone().unwrap();
        let sealed_with = |access: &str, refresh: &str| {
            let mut object = serde_json::Map::new();
            object.insert("client_secret".to_owned(), serde_json::json!("secret"));
            object.insert("access_token".to_owned(), serde_json::json!(access));
            object.insert("refresh_token".to_owned(), serde_json::json!(refresh));
            seal_object(&cipher, &object).unwrap()
        };
        let old = sealed_with("AT-OLD", "RT-OLD");
        sqlx::query!(
            "UPDATE automation.credential SET data_nonce = $2, data_ciphertext = $3 WHERE id = $1",
            credential.id,
            old.nonce,
            old.ciphertext
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut refresh = hold_the_row(&pool, credential.id).await;
        let patching = tokio::spawn({
            let usecase = usecase.clone();
            async move {
                usecase
                    .update_credential(patch(
                        org_id,
                        credential.id,
                        serde_json::json!({ "scopes": "openid email" }),
                    ))
                    .await
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let fresh = sealed_with("AT-FRESH", "RT-ROTATED");
        sqlx::query(
            "UPDATE automation.credential SET data_nonce = $2, data_ciphertext = $3 WHERE id = $1",
        )
        .bind(credential.id)
        .bind(fresh.nonce)
        .bind(fresh.ciphertext)
        .execute(&mut *refresh)
        .await
        .unwrap();
        refresh.commit().await.unwrap();
        let saved = patching.await.unwrap().unwrap();

        let sealed = sealed_object(&pool, &usecase, credential.id).await;
        assert_eq!(sealed["access_token"], serde_json::json!("AT-FRESH"));
        assert_eq!(sealed["refresh_token"], serde_json::json!("RT-ROTATED"));
        assert_eq!(saved.data["scopes"], serde_json::json!("openid email"));
    }
}
