use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Expense, ExpenseId, OrganizationId, PaymentMethod,
    domain::expense::ports::ExpenseRepository,
    infrastructure::{
        expense::postgres::model::ExpenseRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Expense, backend = Postgres)]
pub struct PgExpenseRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgExpenseRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ExpenseRepository for PgExpenseRepository<'tx> {
    async fn insert(&mut self, expense: &Expense) -> Result<Expense, CoreError> {
        {
            let mut tx = self.tx.lock().await;
            sqlx::query!(
                r#"
                INSERT INTO expenses (id, org_id, date, label, supplier_id, category_id, reference, amount_ht_cents, vat_cents, due_date, status, paid_on, method, advanced_by_member_id, reimbursed_on, customer_id, proof_file_key, note, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                "#,
                expense.id.0,
                expense.organization_id.0,
                expense.date,
                expense.label,
                expense.supplier_id.map(|id| id.0),
                expense.category_id.0,
                expense.reference,
                expense.amount_ht_cents,
                expense.vat_cents,
                expense.due_date,
                expense.status.as_str(),
                expense.paid_on,
                expense.method.map(PaymentMethod::as_str),
                expense.advanced_by_member_id.map(|id| id.0),
                expense.reimbursed_on,
                expense.customer_id.map(|id| id.0),
                expense.proof_file_key,
                expense.note,
                expense.created_at,
                expense.updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        self.find_by_id(expense.id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn find_by_id(&mut self, id: ExpenseId) -> Result<Option<Expense>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExpenseRow,
            r#"
            SELECT e.id, e.org_id, e.date, e.label, e.supplier_id, e.category_id, e.reference, e.amount_ht_cents, e.vat_cents, e.due_date, e.status, e.paid_on, e.method, e.advanced_by_member_id, e.reimbursed_on, e.customer_id, e.proof_file_key, e.note, e.created_at, e.updated_at,
                s.name AS "supplier_name?", c.name AS "customer_name?"
            FROM expenses e
            LEFT JOIN suppliers s ON s.id = e.supplier_id
            LEFT JOIN customers c ON c.id = e.customer_id
            WHERE e.id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Expense>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ExpenseRow,
            r#"
            SELECT e.id, e.org_id, e.date, e.label, e.supplier_id, e.category_id, e.reference, e.amount_ht_cents, e.vat_cents, e.due_date, e.status, e.paid_on, e.method, e.advanced_by_member_id, e.reimbursed_on, e.customer_id, e.proof_file_key, e.note, e.created_at, e.updated_at,
                s.name AS "supplier_name?", c.name AS "customer_name?"
            FROM expenses e
            LEFT JOIN suppliers s ON s.id = e.supplier_id
            LEFT JOIN customers c ON c.id = e.customer_id
            WHERE e.org_id = $1
            ORDER BY e.date DESC, e.id ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn update(&mut self, expense: &Expense) -> Result<Expense, CoreError> {
        {
            let mut tx = self.tx.lock().await;
            let result = sqlx::query!(
                r#"
                UPDATE expenses
                SET date = $2, label = $3, supplier_id = $4, category_id = $5, reference = $6, amount_ht_cents = $7, vat_cents = $8, due_date = $9, customer_id = $10, proof_file_key = $11, note = $12, updated_at = $13
                WHERE id = $1
                "#,
                expense.id.0,
                expense.date,
                expense.label,
                expense.supplier_id.map(|id| id.0),
                expense.category_id.0,
                expense.reference,
                expense.amount_ht_cents,
                expense.vat_cents,
                expense.due_date,
                expense.customer_id.map(|id| id.0),
                expense.proof_file_key,
                expense.note,
                expense.updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            if result.rows_affected() == 0 {
                return Err(CoreError::NotFound);
            }
        }

        self.find_by_id(expense.id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn mark_paid(
        &mut self,
        id: ExpenseId,
        paid_on: NaiveDate,
        method: PaymentMethod,
        updated_at: DateTime<Utc>,
    ) -> Result<Option<Expense>, CoreError> {
        let changed = {
            let mut tx = self.tx.lock().await;
            sqlx::query!(
                r#"
                UPDATE expenses
                SET status = 'PAID', paid_on = $2, method = $3, updated_at = $4
                WHERE id = $1 AND status = 'TO_PAY' AND advanced_by_member_id IS NULL
                "#,
                id.0,
                paid_on,
                method.as_str(),
                updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .rows_affected()
        };

        if changed == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn reimburse(
        &mut self,
        id: ExpenseId,
        reimbursed_on: NaiveDate,
        updated_at: DateTime<Utc>,
    ) -> Result<Option<Expense>, CoreError> {
        let changed = {
            let mut tx = self.tx.lock().await;
            sqlx::query!(
                r#"
                UPDATE expenses
                SET reimbursed_on = $2, updated_at = $3
                WHERE id = $1 AND advanced_by_member_id IS NOT NULL AND reimbursed_on IS NULL
                "#,
                id.0,
                reimbursed_on,
                updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .rows_affected()
        };

        if changed == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&mut self, id: ExpenseId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM expenses WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
