use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerId, Deal, DealId, LeadSource, LostReason, OrganizationId, StageTotal,
    domain::deal::{commands::DealFilter, ports::DealRepository},
    infrastructure::{
        deal::postgres::model::{DealRow, StageTotalRow},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Deal, backend = Postgres)]
pub struct PgDealRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDealRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn like_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

impl<'tx> DealRepository for PgDealRepository<'tx> {
    async fn insert(&mut self, deal: &Deal) -> Result<Deal, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DealRow,
            r#"
            INSERT INTO deals (id, org_id, customer_id, title, amount_cents, stage, expected_close_date, owner_member_id, source, lost_reason, notes, created_at, stage_changed_at, closed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, org_id, customer_id, title, amount_cents, stage, expected_close_date, owner_member_id, source, lost_reason, notes, created_at, stage_changed_at, closed_at,
                (SELECT COUNT(*) FROM quotes q WHERE q.deal_id = deals.id AND q.deleted_at IS NULL) AS "quote_count!"
            "#,
            deal.id.0,
            deal.organization_id.0,
            deal.customer_id.0,
            deal.title.as_str(),
            deal.amount.get(),
            deal.stage.name().as_str(),
            deal.expected_close_date,
            deal.owner_member_id.map(|id| id.0),
            deal.source.map(LeadSource::as_str),
            deal.stage.lost_reason().map(LostReason::as_str),
            deal.notes,
            deal.created_at,
            deal.stage_changed_at,
            deal.closed_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: DealId) -> Result<Option<Deal>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DealRow,
            r#"
            SELECT id, org_id, customer_id, title, amount_cents, stage, expected_close_date, owner_member_id, source, lost_reason, notes, created_at, stage_changed_at, closed_at,
                (SELECT COUNT(*) FROM quotes q WHERE q.deal_id = deals.id AND q.deleted_at IS NULL) AS "quote_count!"
            FROM deals
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list(
        &mut self,
        filter: &DealFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Deal>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let customer_id = filter.customer_id.map(|id| id.0);
        let stage = filter.stage.map(|stage| stage.as_str());
        let pattern = filter.query.as_deref().map(like_pattern);

        let rows = sqlx::query_as!(
            DealRow,
            r#"
            SELECT d.id, d.org_id, d.customer_id, d.title, d.amount_cents, d.stage, d.expected_close_date, d.owner_member_id, d.source, d.lost_reason, d.notes, d.created_at, d.stage_changed_at, d.closed_at,
                (SELECT COUNT(*) FROM quotes q WHERE q.deal_id = d.id AND q.deleted_at IS NULL) AS "quote_count!"
            FROM deals d
            JOIN customers c ON c.id = d.customer_id
            WHERE d.org_id = $1
              AND ($2::uuid IS NULL OR d.customer_id = $2)
              AND ($3::text IS NULL OR d.stage = $3)
              AND ($4::text IS NULL OR d.title ILIKE $4 OR c.name ILIKE $4)
            ORDER BY d.stage_changed_at DESC, d.id DESC
            LIMIT $5 OFFSET $6
            "#,
            filter.organization_id.0,
            customer_id,
            stage,
            pattern,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM deals d
            JOIN customers c ON c.id = d.customer_id
            WHERE d.org_id = $1
              AND ($2::uuid IS NULL OR d.customer_id = $2)
              AND ($3::text IS NULL OR d.stage = $3)
              AND ($4::text IS NULL OR d.title ILIKE $4 OR c.name ILIKE $4)
            "#,
            filter.organization_id.0,
            customer_id,
            stage,
            pattern,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let deals = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((deals, total as u64))
    }

    async fn update(&mut self, deal: &Deal) -> Result<Deal, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DealRow,
            r#"
            UPDATE deals
            SET customer_id = $2, title = $3, amount_cents = $4, stage = $5, expected_close_date = $6,
                owner_member_id = $7, source = $8, lost_reason = $9, notes = $10,
                stage_changed_at = $11, closed_at = $12
            WHERE id = $1
            RETURNING id, org_id, customer_id, title, amount_cents, stage, expected_close_date, owner_member_id, source, lost_reason, notes, created_at, stage_changed_at, closed_at,
                (SELECT COUNT(*) FROM quotes q WHERE q.deal_id = deals.id AND q.deleted_at IS NULL) AS "quote_count!"
            "#,
            deal.id.0,
            deal.customer_id.0,
            deal.title.as_str(),
            deal.amount.get(),
            deal.stage.name().as_str(),
            deal.expected_close_date,
            deal.owner_member_id.map(|id| id.0),
            deal.source.map(LeadSource::as_str),
            deal.stage.lost_reason().map(LostReason::as_str),
            deal.notes,
            deal.stage_changed_at,
            deal.closed_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, id: DealId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"UPDATE quotes SET deal_id = NULL WHERE deal_id = $1 AND deleted_at IS NOT NULL"#,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let result = sqlx::query!(r#"DELETE FROM deals WHERE id = $1"#, id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn lose_open_of_customer(
        &mut self,
        customer_id: CustomerId,
        reason: &LostReason,
        now: DateTime<Utc>,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE deals
            SET stage = 'LOST', lost_reason = $2, stage_changed_at = $3, closed_at = $3
            WHERE customer_id = $1 AND stage IN ('NEW', 'VISIT', 'QUOTE_SENT', 'NEGOTIATION')
            "#,
            customer_id.0,
            reason.as_str(),
            now,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }

    async fn stage_totals(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<StageTotal>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            StageTotalRow,
            r#"
            SELECT stage AS "stage!", COUNT(*) AS "count!", COALESCE(SUM(amount_cents), 0)::bigint AS "amount_cents!"
            FROM deals
            WHERE org_id = $1
            GROUP BY stage
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn closed_totals_since(
        &mut self,
        organization_id: OrganizationId,
        since: DateTime<Utc>,
    ) -> Result<Vec<StageTotal>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            StageTotalRow,
            r#"
            SELECT stage AS "stage!", COUNT(*) AS "count!", COALESCE(SUM(amount_cents), 0)::bigint AS "amount_cents!"
            FROM deals
            WHERE org_id = $1 AND stage IN ('WON', 'LOST') AND closed_at >= $2
            GROUP BY stage
            "#,
            organization_id.0,
            since,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}
