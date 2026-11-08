use chrono::NaiveDate;
use common::CoreError;
use mestier_macros::repository;

use crate::{
    DayOverride, MemberId, OrganizationId, PlanningPeriod,
    domain::day_override::ports::DayOverrideRepository,
    infrastructure::{
        day_override::postgres::model::{DayOverrideRow, ShapeColumns},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = DayOverride, backend = Postgres)]
pub struct PgDayOverrideRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDayOverrideRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DayOverrideRepository for PgDayOverrideRepository<'tx> {
    async fn list(
        &mut self,
        organization_id: OrganizationId,
        period: PlanningPeriod,
        member_id: Option<MemberId>,
    ) -> Result<Vec<DayOverride>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            DayOverrideRow,
            r#"
            SELECT id, org_id, member_id, date, kind, start_time, end_time, lunch_start,
                   lunch_end, note, created_at, updated_at
            FROM day_overrides
            WHERE org_id = $1 AND date >= $2 AND date <= $3
              AND ($4::uuid IS NULL OR member_id = $4)
            ORDER BY date, member_id
            "#,
            organization_id.0,
            period.from(),
            period.to(),
            member_id.map(|id| id.0),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(DayOverride::try_from).collect()
    }

    async fn upsert(&mut self, day_override: &DayOverride) -> Result<DayOverride, CoreError> {
        let columns = ShapeColumns::from(&day_override.shape);
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DayOverrideRow,
            r#"
            INSERT INTO day_overrides (
                id, org_id, member_id, date, kind, start_time, end_time, lunch_start,
                lunch_end, note, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (member_id, date) DO UPDATE
            SET kind = EXCLUDED.kind,
                start_time = EXCLUDED.start_time,
                end_time = EXCLUDED.end_time,
                lunch_start = EXCLUDED.lunch_start,
                lunch_end = EXCLUDED.lunch_end,
                note = EXCLUDED.note,
                updated_at = EXCLUDED.updated_at
            RETURNING id, org_id, member_id, date, kind, start_time, end_time, lunch_start,
                      lunch_end, note, created_at, updated_at
            "#,
            day_override.id.0,
            day_override.organization_id.0,
            day_override.member_id.0,
            day_override.date,
            day_override.shape.kind().as_str(),
            columns.start_time,
            columns.end_time,
            columns.lunch_start,
            columns.lunch_end,
            day_override.note,
            day_override.created_at,
            day_override.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        DayOverride::try_from(row)
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        date: NaiveDate,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM day_overrides WHERE org_id = $1 AND member_id = $2 AND date = $3",
            organization_id.0,
            member_id.0,
            date,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() > 0)
    }
}
