use common::CoreError;
use mestier_macros::repository;

use crate::{
    ExpenseCategory, ExpenseCategoryId, OrganizationId,
    domain::expense::ports::ExpenseCategoryRepository,
    infrastructure::{
        expense::postgres::model::ExpenseCategoryRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = ExpenseCategory, backend = Postgres)]
pub struct PgExpenseCategoryRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgExpenseCategoryRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ExpenseCategoryRepository for PgExpenseCategoryRepository<'tx> {
    async fn insert(&mut self, category: &ExpenseCategory) -> Result<ExpenseCategory, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExpenseCategoryRow,
            r#"
            INSERT INTO expense_categories (id, org_id, name, hue, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, name, hue, created_at, updated_at
            "#,
            category.id.0,
            category.organization_id.0,
            category.name,
            category.hue,
            category.created_at,
            category.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(
        &mut self,
        id: ExpenseCategoryId,
    ) -> Result<Option<ExpenseCategory>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExpenseCategoryRow,
            r#"
            SELECT id, org_id, name, hue, created_at, updated_at
            FROM expense_categories
            WHERE id = $1
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
        organization_id: OrganizationId,
    ) -> Result<Vec<ExpenseCategory>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ExpenseCategoryRow,
            r#"
            SELECT id, org_id, name, hue, created_at, updated_at
            FROM expense_categories
            WHERE org_id = $1
            ORDER BY created_at ASC, id ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&mut self, category: &ExpenseCategory) -> Result<ExpenseCategory, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExpenseCategoryRow,
            r#"
            UPDATE expense_categories
            SET name = $2, hue = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, org_id, name, hue, created_at, updated_at
            "#,
            category.id.0,
            category.name,
            category.hue,
            category.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, id: ExpenseCategoryId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM expense_categories WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn is_used(&mut self, id: ExpenseCategoryId) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let used = sqlx::query_scalar!(
            r#"SELECT (
                EXISTS (SELECT 1 FROM expenses WHERE category_id = $1)
                OR EXISTS (SELECT 1 FROM suppliers WHERE default_category_id = $1)
            ) AS "used!""#,
            id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(used)
    }
}
