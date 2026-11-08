use common::CoreError;
use mestier_macros::repository;

use crate::{
    LeaveAccountId, LeaveAdjustment, LeaveAdjustmentId, MemberId, OrganizationId,
    domain::leave_adjustment::ports::LeaveAdjustmentRepository,
    infrastructure::{
        leave_adjustment::postgres::model::LeaveAdjustmentRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = LeaveAdjustment, backend = Postgres)]
pub struct PgLeaveAdjustmentRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgLeaveAdjustmentRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> LeaveAdjustmentRepository for PgLeaveAdjustmentRepository<'tx> {
    async fn list(
        &mut self,
        organization_id: OrganizationId,
        member_id: Option<MemberId>,
        account_id: Option<LeaveAccountId>,
    ) -> Result<Vec<LeaveAdjustment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            LeaveAdjustmentRow,
            r#"
            SELECT id, org_id, member_id, account_id, date, days_hundredths, kind, reason,
                   author_member_id, created_at
            FROM leave_adjustments
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR member_id = $2)
              AND ($3::uuid IS NULL OR account_id = $3)
            ORDER BY date, created_at, id
            "#,
            organization_id.0,
            member_id.map(|id| id.0),
            account_id.map(|id| id.0),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(LeaveAdjustment::try_from).collect()
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAdjustmentId,
    ) -> Result<Option<LeaveAdjustment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LeaveAdjustmentRow,
            r#"
            SELECT id, org_id, member_id, account_id, date, days_hundredths, kind, reason,
                   author_member_id, created_at
            FROM leave_adjustments
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(LeaveAdjustment::try_from).transpose()
    }

    async fn insert(&mut self, adjustment: &LeaveAdjustment) -> Result<LeaveAdjustment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LeaveAdjustmentRow,
            r#"
            INSERT INTO leave_adjustments (
                id, org_id, member_id, account_id, date, days_hundredths, kind, reason,
                author_member_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, org_id, member_id, account_id, date, days_hundredths, kind, reason,
                      author_member_id, created_at
            "#,
            adjustment.id.0,
            adjustment.organization_id.0,
            adjustment.member_id.0,
            adjustment.account_id.0,
            adjustment.date,
            adjustment.days.days().hundredths(),
            adjustment.kind.as_str(),
            adjustment.reason.as_str(),
            adjustment.author_member_id.map(|id| id.0),
            adjustment.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        LeaveAdjustment::try_from(row)
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAdjustmentId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "DELETE FROM leave_adjustments WHERE org_id = $1 AND id = $2",
            organization_id.0,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
