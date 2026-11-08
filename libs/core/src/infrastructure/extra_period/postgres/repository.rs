use common::CoreError;
use mestier_macros::repository;

use crate::{
    ExtraPeriod, ExtraPeriodFilter, ExtraPeriodId, OrganizationId,
    domain::extra_period::ports::ExtraPeriodRepository,
    infrastructure::{
        extra_period::postgres::model::{ExtraPeriodRow, HoursColumns},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = ExtraPeriod, backend = Postgres)]
pub struct PgExtraPeriodRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgExtraPeriodRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ExtraPeriodRepository for PgExtraPeriodRepository<'tx> {
    async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &ExtraPeriodFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ExtraPeriod>, u64), CoreError> {
        let member = filter.member_id.map(|id| id.0);
        let status = filter.status.map(|status| status.as_str());
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM extra_periods
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR member_id = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::date IS NULL OR date >= $4)
              AND ($5::date IS NULL OR date <= $5)
            "#,
            organization_id.0,
            member,
            status,
            filter.from,
            filter.to,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            ExtraPeriodRow,
            r#"
            SELECT id, org_id, member_id, date, start_time, end_time, lunch_start, lunch_end,
                   reason, surcharge_id, rate_bp, status, created_by_member_id,
                   decided_by_member_id, decided_at, decision_comment, created_at, updated_at
            FROM extra_periods
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR member_id = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::date IS NULL OR date >= $4)
              AND ($5::date IS NULL OR date <= $5)
            ORDER BY date DESC, start_time DESC, id
            LIMIT $6 OFFSET $7
            "#,
            organization_id.0,
            member,
            status,
            filter.from,
            filter.to,
            i64::try_from(limit).unwrap_or(i64::MAX),
            i64::try_from(offset).unwrap_or(i64::MAX),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let periods = rows
            .into_iter()
            .map(ExtraPeriod::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((periods, u64::try_from(total).unwrap_or(0)))
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
    ) -> Result<Option<ExtraPeriod>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExtraPeriodRow,
            r#"
            SELECT id, org_id, member_id, date, start_time, end_time, lunch_start, lunch_end,
                   reason, surcharge_id, rate_bp, status, created_by_member_id,
                   decided_by_member_id, decided_at, decision_comment, created_at, updated_at
            FROM extra_periods
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(ExtraPeriod::try_from).transpose()
    }

    async fn insert(&mut self, period: &ExtraPeriod) -> Result<ExtraPeriod, CoreError> {
        let hours = HoursColumns::from(&period.hours);
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExtraPeriodRow,
            r#"
            INSERT INTO extra_periods (
                id, org_id, member_id, date, start_time, end_time, lunch_start, lunch_end,
                reason, surcharge_id, rate_bp, status, created_by_member_id,
                decided_by_member_id, decided_at, decision_comment, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING id, org_id, member_id, date, start_time, end_time, lunch_start, lunch_end,
                      reason, surcharge_id, rate_bp, status, created_by_member_id,
                      decided_by_member_id, decided_at, decision_comment, created_at, updated_at
            "#,
            period.id.0,
            period.organization_id.0,
            period.member_id.0,
            period.date,
            hours.start_time,
            hours.end_time,
            hours.lunch_start,
            hours.lunch_end,
            period.reason.as_str(),
            period.surcharge_id.map(|id| id.0),
            period.rate.get(),
            period.status.as_str(),
            period.created_by_member_id.map(|id| id.0),
            period.decided_by_member_id.map(|id| id.0),
            period.decided_at,
            period.decision_comment,
            period.created_at,
            period.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        ExtraPeriod::try_from(row)
    }

    async fn update(&mut self, period: &ExtraPeriod) -> Result<ExtraPeriod, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ExtraPeriodRow,
            r#"
            UPDATE extra_periods
            SET status = $3, decided_by_member_id = $4, decided_at = $5,
                decision_comment = $6, updated_at = $7
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, member_id, date, start_time, end_time, lunch_start, lunch_end,
                      reason, surcharge_id, rate_bp, status, created_by_member_id,
                      decided_by_member_id, decided_at, decision_comment, created_at, updated_at
            "#,
            period.organization_id.0,
            period.id.0,
            period.status.as_str(),
            period.decided_by_member_id.map(|id| id.0),
            period.decided_at,
            period.decision_comment,
            period.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.ok_or(CoreError::NotFound)
            .and_then(ExtraPeriod::try_from)
    }
}
