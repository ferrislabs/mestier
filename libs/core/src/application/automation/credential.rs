use std::sync::Arc;

use chrono::Utc;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        connector::auth_scheme,
        credential::{
            CreateCredentialCommand, Credential, CredentialError, CredentialOrigin,
            UpdateCredentialCommand, validate_credential_data,
        },
        ports::{CredentialRepository, WorkflowRepository},
    },
    infrastructure::automation::webhook::secret::SecretCipher,
};

impl MestierUseCase {
    /// The key sealing credential data at rest, or the same refusal the
    /// automation worker gives at startup for a missing
    /// `AUTOMATION_SECRET_KEY`: loud and explicit, never a silently stored
    /// plaintext.
    fn require_cipher(&self) -> Result<Arc<SecretCipher>, CoreError> {
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
    ) -> Result<Credential, CoreError> {
        let cipher = self.require_cipher()?;

        let plaintext = match command.origin {
            CredentialOrigin::Supplied => {
                let data = command.data.ok_or_else(|| {
                    CoreError::Conflict("a supplied credential requires data".to_owned())
                })?;
                validate_credential_data(&command.kind, &data)?;
                serde_json::to_vec(&data).map_err(|e| {
                    CoreError::Internal(format!("credential data cannot be serialized: {e}"))
                })?
            }
            CredentialOrigin::Generated => {
                auth_scheme(&command.kind).ok_or_else(|| {
                    CoreError::from(CredentialError::UnknownScheme {
                        kind: command.kind.clone(),
                    })
                })?;
                cipher.generate_secret()?
            }
        };
        let sealed = cipher.seal(&plaintext)?;

        let now = Utc::now();
        let credential = Credential {
            id: generate_uuid_v7(),
            org_id: command.org_id,
            kind: command.kind,
            name: command.name,
            origin: command.origin,
            created_at: now,
            updated_at: now,
        };

        let mut repository = credential_repository;
        repository.insert(&credential, &sealed).await
    }

    #[transactional(credential)]
    pub async fn list_credentials(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<Credential>, CoreError> {
        let mut repository = credential_repository;
        repository.list_by_organization(org_id).await
    }

    #[transactional(credential)]
    pub async fn find_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Credential>, CoreError> {
        let mut repository = credential_repository;
        repository.find_by_id(org_id, id).await
    }

    #[transactional(credential)]
    pub async fn update_credential(
        &self,
        command: UpdateCredentialCommand,
    ) -> Result<Credential, CoreError> {
        let mut repository = credential_repository;
        let existing = repository
            .find_by_id(command.org_id, command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let sealed = match command.data {
            Some(data) => {
                validate_credential_data(&existing.kind, &data)?;
                let cipher = self.require_cipher()?;
                let plaintext = serde_json::to_vec(&data).map_err(|e| {
                    CoreError::Internal(format!("credential data cannot be serialized: {e}"))
                })?;
                Some(cipher.seal(&plaintext)?)
            }
            None => None,
        };

        let updated = Credential {
            name: command.name.unwrap_or(existing.name),
            ..existing
        };

        repository
            .update(command.org_id, &updated, sealed.as_ref())
            .await
    }

    /// Refuses to delete a credential a workflow still references.
    ///
    /// Workflows did not exist when this use case first shipped (#198), so
    /// it deleted without looking at who used the credential. Now #199
    /// gives it a look: the reference lives inside `workflow_version.graph`
    /// (JSONB, no FK to protect it), so this is a domain rule rather than
    /// something the database refuses on its own — see
    /// `WorkflowRepository::workflows_referencing_credential`.
    #[transactional(credential, workflow)]
    pub async fn delete_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let mut credentials = credential_repository;
        let mut workflows = workflow_repository;

        credentials
            .find_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let referencing_workflows = workflows
            .workflows_referencing_credential(org_id, id)
            .await?;
        if !referencing_workflows.is_empty() {
            let names = referencing_workflows
                .iter()
                .map(|workflow| workflow.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(CoreError::Conflict(format!(
                "credential is still referenced by workflow(s): {names}"
            )));
        }

        credentials.delete(org_id, id).await
    }

    /// Regenerates a `Generated` credential's sealed bytes. Refused for a
    /// `Supplied` one: it has nothing of Mestier's own to regenerate, only
    /// the user's own secret, which rotating would silently invalidate.
    #[transactional(credential)]
    pub async fn rotate_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Credential, CoreError> {
        let mut repository = credential_repository;
        let existing = repository
            .find_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        existing.origin.ensure_rotatable()?;

        let cipher = self.require_cipher()?;
        let plaintext = cipher.generate_secret()?;
        let sealed = cipher.seal(&plaintext)?;

        repository.update(org_id, &existing, Some(&sealed)).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::domain::automation::credential::CredentialOrigin;
    use crate::domain::automation::workflow::{CreateWorkflowCommand, Graph, PlacedConnector};
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run credential use case integration tests");
        PgPool::connect(&url).await.unwrap()
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

        let created = usecase
            .create_credential(CreateCredentialCommand {
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
    async fn creating_a_credential_with_a_missing_field_is_refused_and_names_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "invalid").await;
        let usecase = use_case(pool);

        let mut data = odoo_data();
        data.as_object_mut().unwrap().remove("api_key");

        let error = usecase
            .create_credential(CreateCredentialCommand {
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Broken".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(data),
            })
            .await
            .expect_err("a credential missing its key cannot authenticate");

        assert!(format!("{error}").contains("api_key"), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_credential_with_an_unknown_kind_is_refused_and_names_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "unknown-kind").await;
        let usecase = use_case(pool);

        let error = usecase
            .create_credential(CreateCredentialCommand {
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

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

    /// The guard added in #199: a credential a workflow's graph still
    /// references must not disappear out from under it, and the refusal
    /// names the workflow so whoever is deleting it knows what to fix first.
    ///
    /// No connector in the shipped catalogue accepts a credential today
    /// (both `flow.*` descriptors are `AuthRequirement::None`), so
    /// `usecase.save_workflow_version` — which runs `validate_graph` — can
    /// never be made to persist a graph that references one. The guard
    /// itself must not care: it reads `credential_id` out of the graph's
    /// JSON, not the catalogue, so the version is inserted directly through
    /// `PgWorkflowRepository`, the same way the repository-level test
    /// `workflows_referencing_credential_names_the_workflow_whose_graph_contains_it`
    /// does.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_credential_still_referenced_by_a_workflow_is_refused_and_names_it() {
        use crate::infrastructure::automation::postgres::PgWorkflowRepository;
        use crate::infrastructure::postgres::with_tx;

        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-referenced").await;
        let usecase = use_case(pool.clone());

        let created = usecase
            .create_credential(CreateCredentialCommand {
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Token".to_owned(),
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

        let graph = Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "odoo.create_partner".to_string(),
                version: 1,
                credential_id: Some(created.id),
                config: serde_json::Map::new(),
            }],
            edges: Vec::new(),
        };
        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert_version(org_id, workflow.id, &graph, None).await
        })
        .await
        .unwrap();

        let error = usecase
            .delete_credential(org_id, created.id)
            .await
            .expect_err("a referenced credential must not be deletable");

        assert!(matches!(error, CoreError::Conflict(_)));
        assert!(format!("{error}").contains("Uses the token"), "{error}");

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
    async fn deleting_an_unreferenced_credential_still_succeeds() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-unreferenced").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

        let rotated = usecase.rotate_credential(org_id, created.id).await.unwrap();
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

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn rotating_a_supplied_credential_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "rotate-supplied").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_credential(CreateCredentialCommand {
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

        assert!(format!("{error}").contains("never rotated"), "{error}");
    }
}
