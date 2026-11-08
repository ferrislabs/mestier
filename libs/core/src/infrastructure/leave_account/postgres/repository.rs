use common::CoreError;
use mestier_macros::repository;

use crate::{
    LeaveAccount, LeaveAccountId, OrganizationId,
    domain::leave_account::ports::LeaveAccountRepository,
    infrastructure::{
        leave_account::postgres::model::LeaveAccountRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = LeaveAccount, backend = Postgres)]
pub struct PgLeaveAccountRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgLeaveAccountRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> LeaveAccountRepository for PgLeaveAccountRepository<'tx> {
    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<LeaveAccount>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            LeaveAccountRow,
            r#"
            SELECT id, org_id, name, acquisition_kind, acquisition_hundredths,
                   period_start_month, carry_over_max_hundredths, archived_at, created_at
            FROM leave_accounts
            WHERE org_id = $1
            ORDER BY created_at, id
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(LeaveAccount::try_from).collect()
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAccountId,
    ) -> Result<Option<LeaveAccount>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LeaveAccountRow,
            r#"
            SELECT id, org_id, name, acquisition_kind, acquisition_hundredths,
                   period_start_month, carry_over_max_hundredths, archived_at, created_at
            FROM leave_accounts
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(LeaveAccount::try_from).transpose()
    }

    async fn insert(&mut self, account: &LeaveAccount) -> Result<LeaveAccount, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LeaveAccountRow,
            r#"
            INSERT INTO leave_accounts (
                id, org_id, name, acquisition_kind, acquisition_hundredths,
                period_start_month, carry_over_max_hundredths, archived_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, org_id, name, acquisition_kind, acquisition_hundredths,
                      period_start_month, carry_over_max_hundredths, archived_at, created_at
            "#,
            account.id.0,
            account.organization_id.0,
            account.name.as_str(),
            account.acquisition.kind_str(),
            account.acquisition.amount().days().hundredths(),
            i16::from(account.period_start_month.get()),
            account.carry_over_max.days().hundredths(),
            account.archived_at,
            account.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        LeaveAccount::try_from(row)
    }

    async fn update(&mut self, account: &LeaveAccount) -> Result<LeaveAccount, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LeaveAccountRow,
            r#"
            UPDATE leave_accounts
            SET name = $3, acquisition_kind = $4, acquisition_hundredths = $5,
                period_start_month = $6, carry_over_max_hundredths = $7, archived_at = $8
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, acquisition_kind, acquisition_hundredths,
                      period_start_month, carry_over_max_hundredths, archived_at, created_at
            "#,
            account.organization_id.0,
            account.id.0,
            account.name.as_str(),
            account.acquisition.kind_str(),
            account.acquisition.amount().days().hundredths(),
            i16::from(account.period_start_month.get()),
            account.carry_over_max.days().hundredths(),
            account.archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.ok_or(CoreError::NotFound)
            .and_then(LeaveAccount::try_from)
    }
}
