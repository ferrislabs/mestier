use std::str::FromStr;

use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    domain::automation::{
        credential::{Credential, CredentialOrigin, CredentialStatus},
        ports::CredentialRepository,
        secret::SealedSecret,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Credential, backend = Postgres)]
pub struct PgCredentialRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCredentialRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct CredentialRow {
    id: Uuid,
    org_id: Uuid,
    kind: String,
    name: String,
    description: String,
    origin: String,
    status: String,
    account: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    data: Value,
    secret_names: Vec<String>,
    fields_split: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<CredentialRow> for Credential {
    type Error = CoreError;

    fn try_from(row: CredentialRow) -> Result<Self, Self::Error> {
        let origin = CredentialOrigin::from_str(&row.origin).map_err(|e| {
            CoreError::Internal(format!("invalid credential origin in database: {e}"))
        })?;
        let status = CredentialStatus::from_str(&row.status).map_err(|e| {
            CoreError::Internal(format!("invalid credential status in database: {e}"))
        })?;
        let data = match row.data {
            Value::Object(data) => data,
            _ => Map::new(),
        };

        Ok(Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            kind: row.kind,
            name: row.name,
            description: row.description,
            origin,
            status,
            account: row.account,
            expires_at: row.expires_at,
            data,
            secrets_set: row.secret_names,
            fields_split: row.fields_split,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl<'tx> CredentialRepository for PgCredentialRepository<'tx> {
    async fn insert(
        &mut self,
        credential: &Credential,
        sealed: &SealedSecret,
    ) -> Result<Credential, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CredentialRow,
            r#"INSERT INTO automation.credential
                   (id, org_id, kind, name, description, origin, status, account, expires_at,
                    data, secret_names, fields_split, data_nonce, data_ciphertext)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING id, org_id, kind, name, description, origin, status, account, expires_at,
                         data, secret_names, fields_split, created_at, updated_at"#,
            credential.id,
            credential.org_id.0,
            credential.kind,
            credential.name,
            credential.description,
            credential.origin.as_str(),
            credential.status.as_str(),
            credential.account,
            credential.expires_at,
            Value::Object(credential.data.clone()),
            &credential.secrets_set,
            credential.fields_split,
            sealed.nonce,
            sealed.ciphertext,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<Credential>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CredentialRow,
            r#"SELECT id, org_id, kind, name, description, origin, status, account, expires_at,
                      data, secret_names, fields_split, created_at, updated_at
               FROM automation.credential
               WHERE org_id = $1
               ORDER BY kind, name"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Credential::try_from).collect()
    }

    async fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Credential>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CredentialRow,
            r#"SELECT id, org_id, kind, name, description, origin, status, account, expires_at,
                      data, secret_names, fields_split, created_at, updated_at
               FROM automation.credential WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Credential::try_from).transpose()
    }

    async fn find_sealed_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<(Credential, SealedSecret)>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"SELECT id, org_id, kind, name, description, origin, status, account, expires_at,
                      data, secret_names, fields_split, created_at, updated_at,
                      data_nonce, data_ciphertext
               FROM automation.credential WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let credential = Credential::try_from(CredentialRow {
            id: row.id,
            org_id: row.org_id,
            kind: row.kind,
            name: row.name,
            description: row.description,
            origin: row.origin,
            status: row.status,
            account: row.account,
            expires_at: row.expires_at,
            data: row.data,
            secret_names: row.secret_names,
            fields_split: row.fields_split,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })?;
        let sealed = SealedSecret {
            nonce: row.data_nonce,
            ciphertext: row.data_ciphertext,
        };

        Ok(Some((credential, sealed)))
    }

    #[allow(clippy::needless_lifetimes)]
    async fn update<'a>(
        &mut self,
        org_id: OrganizationId,
        credential: &Credential,
        sealed: Option<&'a SealedSecret>,
    ) -> Result<Credential, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CredentialRow,
            r#"UPDATE automation.credential
               SET name = $3,
                   description = $4,
                   status = $5,
                   account = $6,
                   expires_at = $7,
                   data = $8,
                   secret_names = $9,
                   fields_split = $10,
                   data_nonce = COALESCE($11, data_nonce),
                   data_ciphertext = COALESCE($12, data_ciphertext),
                   updated_at = now()
               WHERE org_id = $1 AND id = $2
               RETURNING id, org_id, kind, name, description, origin, status, account, expires_at,
                         data, secret_names, fields_split, created_at, updated_at"#,
            org_id.0,
            credential.id,
            credential.name,
            credential.description,
            credential.status.as_str(),
            credential.account,
            credential.expires_at,
            Value::Object(credential.data.clone()),
            &credential.secrets_set,
            credential.fields_split,
            sealed.map(|s| s.nonce.as_slice()),
            sealed.map(|s| s.ciphertext.as_slice()),
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn delete(&mut self, org_id: OrganizationId, id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"WITH removed AS (
                   DELETE FROM automation.credential
                   WHERE org_id = $1 AND id = $2
                   RETURNING id, org_id, name
               )
               INSERT INTO automation.credential_tombstone (id, org_id, name)
               SELECT id, org_id, name FROM removed
               ON CONFLICT (id) DO NOTHING"#,
            org_id.0,
            id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn list_unsplit(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<(Credential, SealedSecret)>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"SELECT id, org_id, kind, name, description, origin, status, account, expires_at,
                      data, secret_names, fields_split, created_at, updated_at,
                      data_nonce, data_ciphertext
               FROM automation.credential
               WHERE org_id = $1 AND NOT fields_split AND origin = 'supplied'
               ORDER BY id
               FOR UPDATE"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let credential = Credential::try_from(CredentialRow {
                    id: row.id,
                    org_id: row.org_id,
                    kind: row.kind,
                    name: row.name,
                    description: row.description,
                    origin: row.origin,
                    status: row.status,
                    account: row.account,
                    expires_at: row.expires_at,
                    data: row.data,
                    secret_names: row.secret_names,
                    fields_split: row.fields_split,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })?;
                let sealed = SealedSecret {
                    nonce: row.data_nonce,
                    ciphertext: row.data_ciphertext,
                };
                Ok((credential, sealed))
            })
            .collect()
    }

    async fn store_split(
        &mut self,
        org_id: OrganizationId,
        credential: &Credential,
        sealed: &SealedSecret,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"UPDATE automation.credential
               SET data = $3,
                   secret_names = $4,
                   fields_split = TRUE,
                   data_nonce = $5,
                   data_ciphertext = $6
               WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            credential.id,
            Value::Object(credential.data.clone()),
            &credential.secrets_set,
            sealed.nonce,
            sealed.ciphertext,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn usage_counts(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<(Uuid, i64)>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"SELECT c.id AS "id!", COUNT(DISTINCT wv.id)::bigint AS "used_by!"
               FROM automation.credential c
               LEFT JOIN automation.workflow w
                      ON w.org_id = c.org_id AND w.current_version_id IS NOT NULL
               LEFT JOIN automation.workflow_version wv
                      ON wv.id = w.current_version_id
                     AND (
                         wv.graph @> jsonb_build_object(
                             'connectors',
                             jsonb_build_array(jsonb_build_object('credential_id', c.id::text))
                         )
                         OR EXISTS (
                             SELECT 1
                             FROM jsonb_array_elements(wv.graph -> 'triggers') AS trig(value)
                             WHERE trig.value -> 'kind' -> 'Webhook' ->> 'credential_id' = c.id::text
                         )
                         OR EXISTS (
                             SELECT 1
                             FROM jsonb_array_elements(wv.graph -> 'connectors') AS node(value)
                             WHERE node.value -> 'config' ->> 'signing_credential_id' = c.id::text
                         )
                     )
               WHERE c.org_id = $1
               GROUP BY c.id"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|row| (row.id, row.used_by)).collect())
    }

    async fn count_locking_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<i64, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!("credential-quota:{}", org_id.0))
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM automation.credential WHERE org_id = $1"#,
            org_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn find_sealed_for_update(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<(Credential, SealedSecret)>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("SET LOCAL lock_timeout = '2s'")
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        let row = sqlx::query!(
            r#"SELECT id, org_id, kind, name, description, origin, status, account, expires_at,
                      data, secret_names, fields_split, created_at, updated_at,
                      data_nonce, data_ciphertext
               FROM automation.credential WHERE org_id = $1 AND id = $2
               FOR UPDATE"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let credential = Credential::try_from(CredentialRow {
            id: row.id,
            org_id: row.org_id,
            kind: row.kind,
            name: row.name,
            description: row.description,
            origin: row.origin,
            status: row.status,
            account: row.account,
            expires_at: row.expires_at,
            data: row.data,
            secret_names: row.secret_names,
            fields_split: row.fields_split,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })?;
        let sealed = SealedSecret {
            nonce: row.data_nonce,
            ciphertext: row.data_ciphertext,
        };

        Ok(Some((credential, sealed)))
    }

    async fn store_tokens(
        &mut self,
        org_id: OrganizationId,
        credential: &Credential,
        sealed: &SealedSecret,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"UPDATE automation.credential
               SET status = $3,
                   account = $4,
                   expires_at = $5,
                   secret_names = $6,
                   data_nonce = $7,
                   data_ciphertext = $8
               WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            credential.id,
            credential.status.as_str(),
            credential.account,
            credential.expires_at,
            &credential.secrets_set,
            sealed.nonce,
            sealed.ciphertext,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn deleted_name(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<String>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar!(
            "SELECT name FROM automation.credential_tombstone WHERE org_id = $1 AND id = $2",
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::credential::CredentialOrigin;
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::postgres::with_tx;

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

    fn cipher() -> SecretCipher {
        SecretCipher::new(&[7u8; 32]).unwrap()
    }

    fn credential(org_id: OrganizationId, kind: &str, name: &str) -> Credential {
        Credential {
            origin: CredentialOrigin::Supplied,
            ..Credential::fixture(org_id, kind, name)
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_inserted_credential_is_found_by_id_in_its_own_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "insert").await;
        let sealed = cipher().seal(b"probe-secret").unwrap();
        let written = credential(org_id, "bearer_token", "Primary token");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        assert_eq!(inserted.id, written.id);
        assert_eq!(inserted.org_id, org_id);
        assert_eq!(inserted.kind, "bearer_token");
        assert_eq!(inserted.name, "Primary token");
        assert_eq!(inserted.origin, CredentialOrigin::Supplied);

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.find_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, Some(inserted));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_sealed_by_id_returns_the_credential_with_its_sealed_bytes() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "sealed").await;
        let sealed = cipher().seal(b"probe-secret").unwrap();
        let written = credential(org_id, "bearer_token", "Sealed token");

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.find_sealed_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        let (credential, found_sealed) = found.expect("the inserted row is found");
        assert_eq!(credential.id, written.id);
        assert_eq!(credential.name, "Sealed token");
        assert_eq!(found_sealed.nonce, sealed.nonce);
        assert_eq!(found_sealed.ciphertext, sealed.ciphertext);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_sealed_by_id_from_another_organization_reads_back_absent() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "sealed-owner").await;
        let stranger_org = seed_organization(&pool, "sealed-stranger").await;
        let sealed = cipher().seal(b"probe-secret").unwrap();
        let written = credential(org_id, "bearer_token", "Owner's token");

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.find_sealed_by_id(stranger_org, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "owner").await;
        let stranger_org = seed_organization(&pool, "stranger").await;
        let sealed = cipher().seal(b"probe-secret").unwrap();
        let written = credential(org_id, "bearer_token", "Owner's token");

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.find_by_id(stranger_org, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(
            found, None,
            "a credential id from another organization must read back as absent, not forbidden"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_only_returns_the_requesting_organizations_credentials() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "listed").await;
        let other_org = seed_organization(&pool, "not-listed").await;
        let sealed = cipher().seal(b"probe-secret").unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&credential(org_id, "bearer_token", "A"), &sealed)
                .await?;
            repo.insert(&credential(org_id, "http_basic", "B"), &sealed)
                .await?;
            repo.insert(&credential(other_org, "bearer_token", "Not mine"), &sealed)
                .await
        })
        .await
        .unwrap();

        let listed = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.list_by_organization(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|c| c.org_id == org_id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_can_rename_without_touching_the_sealed_bytes() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "rename").await;
        let sealed = cipher().seal(b"original-secret").unwrap();
        let written = credential(org_id, "bearer_token", "Old name");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        let renamed = Credential {
            name: "New name".to_owned(),
            ..inserted.clone()
        };

        let updated = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.update(org_id, &renamed, None).await
        })
        .await
        .unwrap();

        assert_eq!(updated.name, "New name");

        let (nonce, ciphertext): (Vec<u8>, Vec<u8>) = sqlx::query!(
            "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            written.id,
        )
        .fetch_one(&pool)
        .await
        .map(|row| (row.data_nonce, row.data_ciphertext))
        .unwrap();

        assert_eq!(
            nonce, sealed.nonce,
            "a rename must not touch the sealed bytes"
        );
        assert_eq!(ciphertext, sealed.ciphertext);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_with_new_sealed_bytes_replaces_them() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "reseal").await;
        let original = cipher().seal(b"original-secret").unwrap();
        let written = credential(org_id, "bearer_token", "Name");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &original).await
        })
        .await
        .unwrap();

        let replacement = cipher().seal(b"replacement-secret").unwrap();
        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.update(org_id, &inserted, Some(&replacement)).await
        })
        .await
        .unwrap();

        let (nonce, ciphertext): (Vec<u8>, Vec<u8>) = sqlx::query!(
            "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            written.id,
        )
        .fetch_one(&pool)
        .await
        .map(|row| (row.data_nonce, row.data_ciphertext))
        .unwrap();

        assert_eq!(nonce, replacement.nonce);
        assert_eq!(ciphertext, replacement.ciphertext);
        assert_ne!(nonce, original.nonce);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_credential_from_another_organization_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "target").await;
        let stranger_org = seed_organization(&pool, "attacker").await;
        let sealed = cipher().seal(b"probe").unwrap();
        let written = credential(org_id, "bearer_token", "Name");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        let tampered = Credential {
            name: "Renamed by a stranger".to_owned(),
            ..inserted
        };

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.update(stranger_org, &tampered, None).await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));

        let name = sqlx::query_scalar!(
            "SELECT name FROM automation.credential WHERE id = $1",
            written.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(name, "Name", "the row must be untouched");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_removes_the_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete").await;
        let sealed = cipher().seal(b"probe").unwrap();
        let written = credential(org_id, "bearer_token", "Name");

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.delete(org_id, written.id).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.find_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_from_another_organization_leaves_the_row_in_place() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "victim").await;
        let stranger_org = seed_organization(&pool, "intruder").await;
        let sealed = cipher().seal(b"probe").unwrap();
        let written = credential(org_id, "bearer_token", "Name");

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&written, &sealed).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.delete(stranger_org, written.id).await
        })
        .await
        .unwrap();

        let still_there = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.find_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        assert!(
            still_there.is_some(),
            "a delete scoped to another organization must not remove the row"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_same_name_and_kind_twice_in_one_organization_conflicts() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "unique").await;
        let sealed = cipher().seal(b"probe").unwrap();

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&credential(org_id, "bearer_token", "Shared name"), &sealed)
                .await?;
            repo.insert(&credential(org_id, "bearer_token", "Shared name"), &sealed)
                .await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_same_name_in_two_organizations_does_not_conflict() {
        let pool = make_pool().await;
        let org_a = seed_organization(&pool, "org-a").await;
        let org_b = seed_organization(&pool, "org-b").await;
        let sealed = cipher().seal(b"probe").unwrap();

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgCredentialRepository::new(&tx);
            repo.insert(&credential(org_a, "bearer_token", "Shared name"), &sealed)
                .await?;
            repo.insert(&credential(org_b, "bearer_token", "Shared name"), &sealed)
                .await
        })
        .await;

        assert!(outcome.is_ok());
    }

    /// Structural proof that exactly one `SealedSecret` exists in the crate:
    /// the cipher (infrastructure) produces a value that satisfies the
    /// port's parameter (domain) with no conversion at all.
    #[test]
    fn the_cipher_and_the_port_exchange_the_same_sealed_secret_type() {
        fn accepts_the_ports_sealed_secret(sealed: &SealedSecret) -> usize {
            sealed.ciphertext.len()
        }

        let sealed = cipher().seal(b"probe").unwrap();

        assert!(accepts_the_ports_sealed_secret(&sealed) > 0);
    }
}
