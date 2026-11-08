use std::collections::HashMap;

use common::{CoreError, RoleId, generate_uuid_v7};
use discord::{CategoryAccess, CategoryAccessRepository, CategoryId, OrganizationId, UserId};
use mestier_macros::repository;

use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = CategoryAccess, backend = Postgres)]
pub struct PgCategoryAccessRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCategoryAccessRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CategoryAccessRepository for PgCategoryAccessRepository<'tx> {
    async fn replace(
        &mut self,
        organization_id: OrganizationId,
        category_id: CategoryId,
        access: &CategoryAccess,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "DELETE FROM chat.category_access WHERE category_id = $1",
            category_id.0
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for role in &access.roles {
            sqlx::query!(
                "INSERT INTO chat.category_access (id, category_id, org_id, role_id) VALUES ($1, $2, $3, $4)",
                generate_uuid_v7(),
                category_id.0,
                organization_id.0,
                role.0,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        for user in &access.users {
            sqlx::query!(
                "INSERT INTO chat.category_access (id, category_id, org_id, user_id) VALUES ($1, $2, $3, $4)",
                generate_uuid_v7(),
                category_id.0,
                organization_id.0,
                user.0,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn find(&mut self, category_id: CategoryId) -> Result<Option<CategoryAccess>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            "SELECT role_id, user_id FROM chat.category_access WHERE category_id = $1",
            category_id.0
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut access = CategoryAccess::default();
        for row in rows {
            if let Some(role) = row.role_id {
                access.roles.push(RoleId(role));
            }
            if let Some(user) = row.user_id {
                access.users.push(UserId(user));
            }
        }
        Ok(Some(access))
    }

    async fn list_for_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<HashMap<CategoryId, CategoryAccess>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            "SELECT category_id, role_id, user_id FROM chat.category_access WHERE org_id = $1",
            organization_id.0
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let mut by_category: HashMap<CategoryId, CategoryAccess> = HashMap::new();
        for row in rows {
            let access = by_category.entry(CategoryId(row.category_id)).or_default();
            if let Some(role) = row.role_id {
                access.roles.push(RoleId(role));
            }
            if let Some(user) = row.user_id {
                access.users.push(UserId(user));
            }
        }
        Ok(by_category)
    }
}
