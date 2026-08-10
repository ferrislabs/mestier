use std::str::FromStr;

use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        credential::{Credential, CredentialOrigin},
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
    origin: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<CredentialRow> for Credential {
    type Error = CoreError;

    fn try_from(row: CredentialRow) -> Result<Self, Self::Error> {
        let origin = CredentialOrigin::from_str(&row.origin).map_err(|e| {
            CoreError::Internal(format!("invalid credential origin in database: {e}"))
        })?;

        Ok(Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            kind: row.kind,
            name: row.name,
            origin,
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
                   (id, org_id, kind, name, origin, data_nonce, data_ciphertext)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, org_id, kind, name, origin, created_at, updated_at"#,
            credential.id,
            credential.org_id.0,
            credential.kind,
            credential.name,
            credential.origin.as_str(),
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
            r#"SELECT id, org_id, kind, name, origin, created_at, updated_at
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
            r#"SELECT id, org_id, kind, name, origin, created_at, updated_at
               FROM automation.credential WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Credential::try_from).transpose()
    }

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
                   data_nonce = COALESCE($4, data_nonce),
                   data_ciphertext = COALESCE($5, data_ciphertext),
                   updated_at = now()
               WHERE org_id = $1 AND id = $2
               RETURNING id, org_id, kind, name, origin, created_at, updated_at"#,
            org_id.0,
            credential.id,
            credential.name,
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
            "DELETE FROM automation.credential WHERE org_id = $1 AND id = $2",
            org_id.0,
            id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::domain::automation::credential::CredentialOrigin;
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run credential integration tests");
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

    fn cipher() -> SecretCipher {
        SecretCipher::new(&[7u8; 32]).unwrap()
    }

    fn credential(org_id: OrganizationId, kind: &str, name: &str) -> Credential {
        let now = chrono::Utc::now();
        Credential {
            id: generate_uuid_v7(),
            org_id,
            kind: kind.to_owned(),
            name: name.to_owned(),
            origin: CredentialOrigin::Supplied,
            created_at: now,
            updated_at: now,
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

        assert_eq!(nonce, sealed.nonce, "a rename must not touch the sealed bytes");
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
