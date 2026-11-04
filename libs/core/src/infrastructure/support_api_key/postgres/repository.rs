use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, SupportApiKey, SupportApiKeyId, SupportApiKeyRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        support_api_key::postgres::model::SupportApiKeyRow,
    },
};

#[repository(domain = SupportApiKey, backend = Postgres)]
pub struct PgSupportApiKeyRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupportApiKeyRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SupportApiKeyRepository for PgSupportApiKeyRepository<'tx> {
    async fn create(&mut self, key: &SupportApiKey) -> Result<SupportApiKey, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupportApiKeyRow,
            r#"
            INSERT INTO support_api_keys (
                id, org_id, name, prefix, secret_hash, created_at, last_used_at, revoked_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, org_id, name, prefix, secret_hash, created_at, last_used_at, revoked_at
            "#,
            key.id.0,
            key.organization_id.0,
            key.name,
            key.prefix,
            key.secret_hash,
            key.created_at,
            key.last_used_at,
            key.revoked_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<SupportApiKey>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            SupportApiKeyRow,
            r#"
            SELECT id, org_id, name, prefix, secret_hash, created_at, last_used_at, revoked_at
            FROM support_api_keys
            WHERE org_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(SupportApiKey::from).collect())
    }

    async fn find_by_prefix(&mut self, prefix: &str) -> Result<Option<SupportApiKey>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupportApiKeyRow,
            r#"
            SELECT k.id, k.org_id, k.name, k.prefix, k.secret_hash, k.created_at,
                   k.last_used_at, k.revoked_at
            FROM support_api_keys k
            JOIN organizations o ON o.id = k.org_id
            WHERE k.prefix = $1 AND o.deleted_at IS NULL
            "#,
            prefix,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(SupportApiKey::from))
    }

    async fn revoke(
        &mut self,
        organization_id: OrganizationId,
        id: SupportApiKeyId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE support_api_keys
            SET revoked_at = COALESCE(revoked_at, now())
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn touch_last_used(&mut self, id: SupportApiKeyId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            UPDATE support_api_keys
            SET last_used_at = now()
            WHERE id = $1
              AND revoked_at IS NULL
              AND (last_used_at IS NULL OR last_used_at < now() - interval '1 minute')
            "#,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, Utc};
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        application::test_support::{dev_pool, now_storable},
        infrastructure::postgres::with_tx,
    };

    async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        let organization_id = generate_uuid_v7();
        let mut guard = tx.lock().await;
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(owner_id)
        .bind(format!("owner-{owner_id}@example.com"))
        .bind(format!("owner-{owner_id}"))
        .bind("Owner")
        .bind(format!("sub-{owner_id}"))
        .execute(&mut ***guard)
        .await
        .unwrap();
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id)
            .bind("Treyve Paysages")
            .bind(format!("org-{organization_id}"))
            .bind(owner_id)
            .execute(&mut ***guard)
            .await
            .unwrap();

        OrganizationId(organization_id)
    }

    fn key(
        organization_id: OrganizationId,
        prefix: &str,
        created_at: DateTime<Utc>,
    ) -> SupportApiKey {
        SupportApiKey {
            id: SupportApiKeyId(generate_uuid_v7()),
            organization_id,
            name: format!("Intake {prefix}"),
            prefix: prefix.to_owned(),
            secret_hash: vec![7; 32],
            created_at,
            last_used_at: None,
            revoked_at: None,
        }
    }

    async fn rolled_back<F>(pool: &PgPool, work: F)
    where
        F: AsyncFnOnce(SharedTx<'_>),
    {
        with_tx(pool, async |tx| {
            work(tx).await;
            Err::<(), _>(CoreError::Internal("rollback".to_owned()))
        })
        .await
        .unwrap_err();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_returns_every_stored_field_including_the_hash_bytes() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let mut input = key(organization_id, "0a1b2c3d", now_storable());
            input.secret_hash = (0..32).collect();

            let stored = repository.create(&input).await.unwrap();

            assert_eq!(stored, input);
            assert_eq!(
                repository.find_by_prefix("0a1b2c3d").await.unwrap(),
                Some(input)
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_duplicate_prefix_is_refused_even_across_organizations() {
        rolled_back(&dev_pool().await, async |tx| {
            let first_organization = seed_organization(&tx).await;
            let second_organization = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            repository
                .create(&key(first_organization, "deadbeef", now_storable()))
                .await
                .unwrap();

            let duplicate = repository
                .create(&key(second_organization, "deadbeef", now_storable()))
                .await;

            assert!(
                matches!(&duplicate, Err(CoreError::Conflict(constraint)) if constraint == "support_api_keys_prefix_key"),
                "{duplicate:?}"
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_by_prefix_looks_across_organizations_and_misses_cleanly() {
        rolled_back(&dev_pool().await, async |tx| {
            let first_organization = seed_organization(&tx).await;
            let second_organization = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let first = key(first_organization, "11111111", now_storable());
            let second = key(second_organization, "22222222", now_storable());
            repository.create(&first).await.unwrap();
            repository.create(&second).await.unwrap();

            assert_eq!(
                repository.find_by_prefix("11111111").await.unwrap(),
                Some(first)
            );
            assert_eq!(
                repository.find_by_prefix("22222222").await.unwrap(),
                Some(second)
            );
            assert_eq!(repository.find_by_prefix("33333333").await.unwrap(), None);
            assert_eq!(repository.find_by_prefix("").await.unwrap(), None);
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_is_scoped_to_the_organization_newest_first_and_keeps_revoked_keys() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let other_organization = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let now = now_storable();
            let oldest = key(organization_id, "00000001", now - Duration::hours(2));
            let mut middle = key(organization_id, "00000002", now - Duration::hours(1));
            middle.revoked_at = Some(now - Duration::minutes(30));
            let newest = key(organization_id, "00000003", now);
            let foreign = key(other_organization, "00000004", now);
            for input in [&middle, &newest, &oldest, &foreign] {
                repository.create(input).await.unwrap();
            }

            let listed = repository.list(organization_id).await.unwrap();

            assert_eq!(listed, vec![newest, middle, oldest]);
            assert_eq!(
                repository.list(other_organization).await.unwrap(),
                vec![foreign]
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn revoke_reports_whether_the_key_is_in_the_organization() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let other_organization = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let input = key(organization_id, "0000000a", now_storable());
            repository.create(&input).await.unwrap();

            assert!(
                !repository
                    .revoke(other_organization, input.id)
                    .await
                    .unwrap()
            );
            assert!(
                !repository
                    .revoke(organization_id, SupportApiKeyId(generate_uuid_v7()))
                    .await
                    .unwrap()
            );
            assert_eq!(
                repository
                    .find_by_prefix("0000000a")
                    .await
                    .unwrap()
                    .unwrap()
                    .revoked_at,
                None
            );

            assert!(repository.revoke(organization_id, input.id).await.unwrap());
            assert!(
                repository
                    .find_by_prefix("0000000a")
                    .await
                    .unwrap()
                    .unwrap()
                    .revoked_at
                    .is_some()
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn revoke_never_moves_an_existing_revoked_at() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let first_revocation = now_storable() - Duration::days(30);
            let mut input = key(organization_id, "0000000b", now_storable());
            input.revoked_at = Some(first_revocation);
            repository.create(&input).await.unwrap();

            assert!(repository.revoke(organization_id, input.id).await.unwrap());

            assert_eq!(
                repository
                    .find_by_prefix("0000000b")
                    .await
                    .unwrap()
                    .unwrap()
                    .revoked_at,
                Some(first_revocation)
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn touch_last_used_stamps_only_that_key() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let touched = key(organization_id, "0000000c", now_storable());
            let untouched = key(organization_id, "0000000d", now_storable());
            repository.create(&touched).await.unwrap();
            repository.create(&untouched).await.unwrap();

            repository.touch_last_used(touched.id).await.unwrap();

            let after = repository
                .find_by_prefix("0000000c")
                .await
                .unwrap()
                .unwrap();
            assert!(
                after
                    .last_used_at
                    .is_some_and(|at| at >= touched.created_at - Duration::minutes(1))
            );
            assert_eq!(
                repository
                    .find_by_prefix("0000000d")
                    .await
                    .unwrap()
                    .unwrap()
                    .last_used_at,
                None
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn touch_last_used_sets_a_never_used_key_once_and_then_holds_for_a_minute() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let never_used = key(organization_id, "00000010", now_storable());
            repository.create(&never_used).await.unwrap();

            repository.touch_last_used(never_used.id).await.unwrap();
            let first = repository
                .find_by_prefix("00000010")
                .await
                .unwrap()
                .unwrap()
                .last_used_at;
            repository.touch_last_used(never_used.id).await.unwrap();
            let second = repository
                .find_by_prefix("00000010")
                .await
                .unwrap()
                .unwrap()
                .last_used_at;

            assert!(first.is_some());
            assert_eq!(second, first);
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn touch_last_used_does_not_move_a_stamp_younger_than_a_minute() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let recently_used_at = now_storable() - Duration::seconds(30);
            let mut recent = key(organization_id, "00000011", now_storable());
            recent.last_used_at = Some(recently_used_at);
            repository.create(&recent).await.unwrap();

            repository.touch_last_used(recent.id).await.unwrap();

            assert_eq!(
                repository
                    .find_by_prefix("00000011")
                    .await
                    .unwrap()
                    .unwrap()
                    .last_used_at,
                Some(recently_used_at)
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn touch_last_used_moves_a_stamp_older_than_a_minute() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let stale_at = now_storable() - Duration::minutes(2);
            let mut stale = key(organization_id, "00000012", now_storable());
            stale.last_used_at = Some(stale_at);
            repository.create(&stale).await.unwrap();

            repository.touch_last_used(stale.id).await.unwrap();

            let moved = repository
                .find_by_prefix("00000012")
                .await
                .unwrap()
                .unwrap()
                .last_used_at
                .unwrap();
            assert!(moved > stale_at + Duration::minutes(1));
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_by_prefix_misses_a_key_of_a_soft_deleted_organization() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let stored = key(organization_id, "0000000e", now_storable());
            repository.create(&stored).await.unwrap();
            assert_eq!(
                repository.find_by_prefix("0000000e").await.unwrap(),
                Some(stored)
            );

            {
                let mut guard = tx.lock().await;
                sqlx::query("UPDATE organizations SET deleted_at = now() WHERE id = $1")
                    .bind(organization_id.0)
                    .execute(&mut ***guard)
                    .await
                    .unwrap();
            }

            assert_eq!(repository.find_by_prefix("0000000e").await.unwrap(), None);
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn touch_last_used_leaves_a_revoked_key_alone() {
        rolled_back(&dev_pool().await, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let mut repository = PgSupportApiKeyRepository::new(&tx);
            let stored = key(organization_id, "0000000f", now_storable());
            repository.create(&stored).await.unwrap();
            repository.revoke(organization_id, stored.id).await.unwrap();

            repository.touch_last_used(stored.id).await.unwrap();

            assert_eq!(
                repository
                    .find_by_prefix("0000000f")
                    .await
                    .unwrap()
                    .unwrap()
                    .last_used_at,
                None
            );
        })
        .await;
    }
}
