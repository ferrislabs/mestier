use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, UserId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    CustomerId, OrganizationId, PaymentMethod, Receipt, ReceiptId,
    domain::ledger::ports::ReceiptRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Receipt, backend = Postgres)]
pub struct PgReceiptRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgReceiptRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct ReceiptSqlRow {
    id: Uuid,
    org_id: Uuid,
    customer_id: Option<Uuid>,
    amount_cents: i32,
    paid_on: NaiveDate,
    method: String,
    reference: Option<String>,
    note: Option<String>,
    recorded_by: Uuid,
    created_at: DateTime<Utc>,
}

impl TryFrom<ReceiptSqlRow> for Receipt {
    type Error = CoreError;

    fn try_from(row: ReceiptSqlRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ReceiptId(row.id),
            organization_id: OrganizationId(row.org_id),
            customer_id: row.customer_id.map(CustomerId),
            amount_cents: row.amount_cents,
            paid_on: row.paid_on,
            method: PaymentMethod::from_str(&row.method).map_err(|error| {
                CoreError::Internal(format!("invalid payment method in database: {error}"))
            })?,
            reference: row.reference,
            note: row.note,
            recorded_by: UserId(row.recorded_by),
            created_at: row.created_at,
        })
    }
}

impl<'tx> ReceiptRepository for PgReceiptRepository<'tx> {
    async fn insert(&mut self, receipt: &Receipt) -> Result<Receipt, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ReceiptSqlRow,
            r#"
            INSERT INTO unallocated_payments (id, org_id, customer_id, amount_cents, paid_on, method, reference, note, recorded_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, org_id, customer_id, amount_cents, paid_on, method, reference, note, recorded_by, created_at
            "#,
            receipt.id.0,
            receipt.organization_id.0,
            receipt.customer_id.map(|id| id.0),
            receipt.amount_cents,
            receipt.paid_on,
            receipt.method.as_str(),
            receipt.reference,
            receipt.note,
            receipt.recorded_by.0,
            receipt.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: ReceiptId) -> Result<Option<Receipt>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ReceiptSqlRow,
            r#"
            SELECT id, org_id, customer_id, amount_cents, paid_on, method, reference, note, recorded_by, created_at
            FROM unallocated_payments
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn delete(&mut self, id: ReceiptId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM unallocated_payments WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
