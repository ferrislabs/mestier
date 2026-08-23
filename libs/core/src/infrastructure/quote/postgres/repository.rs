use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    OrganizationId, Quote, QuoteId, QuoteLine, QuoteStatus,
    domain::quote::ports::QuoteRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        quote::postgres::model::{QuoteLineRow, QuoteRow, vat_breakdown_to_json},
    },
};

#[repository(domain = Quote, backend = Postgres)]
pub struct PgQuoteRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgQuoteRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> QuoteRepository for PgQuoteRepository<'tx> {
    async fn allocate_number(
        &mut self,
        organization_id: OrganizationId,
        prefix: &str,
        year: i32,
    ) -> Result<String, CoreError> {
        let mut tx = self.tx.lock().await;

        // `INSERT ... ON CONFLICT DO UPDATE` takes a row lock on the
        // counter as soon as it detects the conflict, and holds it until
        // this transaction commits or rolls back. A second allocation for
        // the same organization and year — from a genuinely concurrent
        // transaction — blocks on that lock rather than reading a stale
        // `next_number`, which is what makes this safe under contention
        // without a separate `SELECT ... FOR UPDATE` step.
        let next_number: i32 = sqlx::query_scalar!(
            r#"
            INSERT INTO quote_number_counters (organization_id, year, next_number)
            VALUES ($1, $2, 1)
            ON CONFLICT (organization_id, year)
            DO UPDATE SET
                next_number = quote_number_counters.next_number + 1,
                updated_at = now()
            RETURNING next_number
            "#,
            organization_id.0,
            year,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(format!("{prefix}-{year}-{next_number:04}"))
    }

    async fn insert(&mut self, quote: &Quote) -> Result<Quote, CoreError> {
        let mut tx = self.tx.lock().await;
        let vat_breakdown = vat_breakdown_to_json(&quote.vat_breakdown);
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            INSERT INTO quotes (id, org_id, reference, title, customer_id, customer_context_id, status, net_cents, vat_breakdown, gross_cents, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS text)::quote_status, $8, $9, $10, $11, $12, $13)
            RETURNING id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", net_cents, vat_breakdown, gross_cents, deleted_at, created_at, updated_at
            "#,
            quote.id.0,
            quote.organization_id.0,
            quote.reference,
            quote.title,
            quote.customer_id.0,
            quote.customer_context_id.0,
            quote.status.as_str(),
            quote.net_cents,
            vat_breakdown,
            quote.gross_cents,
            quote.deleted_at,
            quote.created_at,
            quote.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        insert_lines(&mut tx, &quote.lines).await?;
        row.into_quote(quote.lines.clone())
    }

    async fn find_by_id(&mut self, id: QuoteId) -> Result<Option<Quote>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            SELECT id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", net_cents, vat_breakdown, gross_cents, deleted_at, created_at, updated_at
            FROM quotes
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => {
                let lines = fetch_lines(&mut tx, id).await?;
                Ok(Some(row.into_quote(lines)?))
            }
            None => Ok(None),
        }
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            QuoteRow,
            r#"
            SELECT id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", net_cents, vat_breakdown, gross_cents, deleted_at, created_at, updated_at
            FROM quotes
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM quotes WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut quotes = Vec::with_capacity(rows.len());
        for row in rows {
            let lines = fetch_lines(&mut tx, QuoteId(row.id)).await?;
            quotes.push(row.into_quote(lines)?);
        }

        Ok((quotes, total as u64))
    }

    async fn update(&mut self, quote: &Quote) -> Result<Quote, CoreError> {
        let mut tx = self.tx.lock().await;
        let vat_breakdown = vat_breakdown_to_json(&quote.vat_breakdown);
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            UPDATE quotes
            SET reference = $2,
                title = $3,
                customer_id = $4,
                customer_context_id = $5,
                status = CAST($6 AS text)::quote_status,
                net_cents = $7,
                vat_breakdown = $8,
                gross_cents = $9,
                updated_at = $10
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", net_cents, vat_breakdown, gross_cents, deleted_at, created_at, updated_at
            "#,
            quote.id.0,
            quote.reference,
            quote.title,
            quote.customer_id.0,
            quote.customer_context_id.0,
            quote.status.as_str(),
            quote.net_cents,
            vat_breakdown,
            quote.gross_cents,
            quote.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;

        sqlx::query!(
            r#"
            UPDATE quote_lines
            SET deleted_at = $2, updated_at = $2
            WHERE quote_id = $1 AND deleted_at IS NULL
            "#,
            quote.id.0,
            quote.updated_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        insert_lines(&mut tx, &quote.lines).await?;
        row.into_quote(quote.lines.clone())
    }

    async fn update_status(
        &mut self,
        id: QuoteId,
        status: QuoteStatus,
        reference: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Result<Quote, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            UPDATE quotes
            SET status = CAST($2 AS text)::quote_status,
                reference = COALESCE($3, reference),
                updated_at = $4
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", net_cents, vat_breakdown, gross_cents, deleted_at, created_at, updated_at
            "#,
            id.0,
            status.as_str(),
            reference,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;
        let lines = fetch_lines(&mut tx, id).await?;
        row.into_quote(lines)
    }

    async fn soft_delete(
        &mut self,
        id: QuoteId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE quotes
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        sqlx::query!(
            r#"
            UPDATE quote_lines
            SET deleted_at = $2, updated_at = $2
            WHERE quote_id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

async fn fetch_lines(
    conn: &mut PgConnection,
    quote_id: QuoteId,
) -> Result<Vec<QuoteLine>, CoreError> {
    let rows = sqlx::query_as!(
        QuoteLineRow,
        r#"
        SELECT id, org_id, quote_id, service_rate_id, label, quantity, unit, unit_price_cents, vat_rate_bp, notes, photo_keys, deleted_at, created_at, updated_at
        FROM quote_lines
        WHERE quote_id = $1 AND deleted_at IS NULL
        ORDER BY created_at ASC, id ASC
        "#,
        quote_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    rows.into_iter().map(TryInto::try_into).collect()
}

async fn insert_lines(conn: &mut PgConnection, lines: &[QuoteLine]) -> Result<(), CoreError> {
    for line in lines {
        sqlx::query!(
            r#"
            INSERT INTO quote_lines (
                id,
                org_id,
                quote_id,
                service_rate_id,
                label,
                quantity,
                unit,
                unit_price_cents,
                vat_rate_bp,
                notes,
                photo_keys,
                deleted_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            line.id.0,
            line.organization_id.0,
            line.quote_id.0,
            line.service_rate_id.map(|id| id.0),
            line.label,
            line.quantity,
            line.unit.as_str(),
            line.unit_price_cents,
            line.vat_rate_bp,
            line.notes,
            &line.photo_keys,
            line.deleted_at,
            line.created_at,
            line.updated_at,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::{dev_pool, purge};
    use crate::infrastructure::postgres::with_tx;

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    async fn cleanup(pool: &PgPool, org_id: OrganizationId, owner_id: uuid::Uuid) {
        purge(
            pool,
            "DELETE FROM quote_number_counters WHERE organization_id = $1",
            org_id.0,
        )
        .await;
        purge(pool, "DELETE FROM organizations WHERE id = $1", org_id.0).await;
        purge(pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn allocate_number_is_gapless_and_sequential_within_one_transaction() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "sequential").await;

        let first = with_tx(&pool, async |tx| {
            let mut repo = PgQuoteRepository::new(&tx);
            repo.allocate_number(org_id, "DEV", 2026).await
        })
        .await
        .unwrap();
        let second = with_tx(&pool, async |tx| {
            let mut repo = PgQuoteRepository::new(&tx);
            repo.allocate_number(org_id, "DEV", 2026).await
        })
        .await
        .unwrap();
        let different_year = with_tx(&pool, async |tx| {
            let mut repo = PgQuoteRepository::new(&tx);
            repo.allocate_number(org_id, "DEV", 2027).await
        })
        .await
        .unwrap();

        assert_eq!(first, "DEV-2026-0001");
        assert_eq!(second, "DEV-2026-0002");
        assert_eq!(
            different_year, "DEV-2027-0001",
            "the counter is per year, not just per organization"
        );

        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        cleanup(&pool, org_id, owner_id).await;
    }

    /// The rule that makes this safe: `INSERT ... ON CONFLICT DO UPDATE`
    /// takes a row lock on the counter as soon as it detects the conflict,
    /// and holds it until the transaction that took it commits or rolls
    /// back. Two allocations for the same organization and year, started at
    /// the same instant from two different connections, must therefore
    /// still land on two different numbers — asserted here against a real
    /// database under real contention, not asserted in a comment.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn concurrent_allocations_for_the_same_organization_and_year_never_collide() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "contention").await;

        let pool_a = pool.clone();
        let pool_b = pool.clone();

        let (a, b) = tokio::join!(
            with_tx(&pool_a, async |tx| {
                let mut repo = PgQuoteRepository::new(&tx);
                repo.allocate_number(org_id, "DEV", 2026).await
            }),
            with_tx(&pool_b, async |tx| {
                let mut repo = PgQuoteRepository::new(&tx);
                repo.allocate_number(org_id, "DEV", 2026).await
            }),
        );

        let mut numbers = [a.unwrap(), b.unwrap()];
        numbers.sort();

        assert_eq!(
            numbers,
            ["DEV-2026-0001".to_owned(), "DEV-2026-0002".to_owned()],
            "two concurrent allocations for the same organization and year must never collide"
        );

        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        cleanup(&pool, org_id, owner_id).await;
    }
}
