use common::CoreError;
use discord::{Category, CategoryId, CategoryRepository, OrganizationId};
use mestier_macros::repository;

use super::model::CategoryRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Category, backend = Postgres)]
pub struct PgCategoryRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCategoryRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CategoryRepository for PgCategoryRepository<'tx> {
    async fn insert(&mut self, c: &Category) -> Result<Category, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CategoryRow,
            r#"
            INSERT INTO categories (id, org_id, name, position, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, name, position, created_at, updated_at
            "#,
            c.id.0,
            c.organization_id.0,
            c.name,
            c.position,
            c.created_at,
            c.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: CategoryId) -> Result<Option<Category>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CategoryRow,
            r#"
            SELECT id, org_id, name, position, created_at, updated_at
            FROM categories WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(Into::into))
    }

    async fn list_by_organization(
        &mut self,
        org: OrganizationId,
    ) -> Result<Vec<Category>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CategoryRow,
            r#"
            SELECT id, org_id, name, position, created_at, updated_at
            FROM categories WHERE org_id = $1 ORDER BY position ASC, id ASC
            "#,
            org.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&mut self, c: &Category) -> Result<Category, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CategoryRow,
            r#"
            UPDATE categories
            SET name = $2, position = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, org_id, name, position, created_at, updated_at
            "#,
            c.id.0,
            c.name,
            c.position,
            c.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, id: CategoryId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM categories WHERE id = $1", id.0,)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::{OrganizationId, generate_uuid_v7};
    use discord::{Category, CategoryId};
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        PgPool::connect("postgres://ferriskey:ferriskey@localhost:5433/mestier")
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn insert_and_find_by_id() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let org_id = {
                let mut guard = tx.lock().await;
                seed_org(*guard).await
            };

            let cat = Category {
                id: CategoryId(generate_uuid_v7()),
                organization_id: org_id,
                name: "General".into(),
                position: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut repo = PgCategoryRepository::new(&tx);
            let inserted = repo.insert(&cat).await.unwrap();
            assert_eq!(inserted.name, "General");
            let found = repo.find_by_id(cat.id).await.unwrap();
            assert!(found.is_some());
            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }

    /// Seeds a user + organization so the FK `categories_org_id_fkey -> organizations(id)`
    /// is satisfied. Mirrors the channel repo test helper.
    async fn seed_org(tx: &mut sqlx::Transaction<'static, sqlx::Postgres>) -> OrganizationId {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("test-{}@example.com", user_id),
            format!("user-{}", user_id),
            "Test User",
            format!("sub-{}", user_id),
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{}", org_id),
            user_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        OrganizationId(org_id)
    }
}
