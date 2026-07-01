use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    LegalMentionTemplateId, OrganizationId, Quote, QuoteId, QuoteLine, QuoteStatus,
    domain::quote::ports::QuoteRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        quote::postgres::model::{QuoteLineRow, QuoteRow},
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
    async fn next_reference(
        &mut self,
        organization_id: OrganizationId,
        year: i32,
    ) -> Result<String, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))"#,
            organization_id.0.to_string(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let prefix = format!("DEV-{year}-");
        let next_number: i32 = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(MAX(substring(reference FROM $2)::INTEGER), 0) + 1 AS "next_number!"
            FROM quotes
            WHERE org_id = $1 AND reference LIKE $3
            "#,
            organization_id.0,
            (prefix.len() + 1) as i32,
            format!("{prefix}%"),
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(format!("{prefix}{next_number:04}"))
    }

    async fn insert(&mut self, quote: &Quote) -> Result<Quote, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            INSERT INTO quotes (id, org_id, reference, title, customer_id, customer_context_id, status, deposit_basis, deposit_value, total_cents, total_ht_cents, total_vat_cents, total_ttc_cents, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS text)::quote_status, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", deposit_basis, deposit_value, total_cents, total_ht_cents, total_vat_cents, total_ttc_cents, deleted_at, created_at, updated_at
            "#,
            quote.id.0,
            quote.organization_id.0,
            quote.reference,
            quote.title,
            quote.customer_id.0,
            quote.customer_context_id.0,
            quote.status.as_str(),
            quote.deposit_basis,
            quote.deposit_value,
            quote.total_cents,
            quote.total_ht_cents,
            quote.total_vat_cents,
            quote.total_ttc_cents,
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
            SELECT id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", deposit_basis, deposit_value, total_cents, total_ht_cents, total_vat_cents, total_ttc_cents, deleted_at, created_at, updated_at
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
            SELECT id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", deposit_basis, deposit_value, total_cents, total_ht_cents, total_vat_cents, total_ttc_cents, deleted_at, created_at, updated_at
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
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            UPDATE quotes
            SET title = $2,
                customer_id = $3,
                customer_context_id = $4,
                status = CAST($5 AS text)::quote_status,
                deposit_basis = $6,
                deposit_value = $7,
                total_cents = $8,
                total_ht_cents = $9,
                total_vat_cents = $10,
                total_ttc_cents = $11,
                updated_at = $12
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", deposit_basis, deposit_value, total_cents, total_ht_cents, total_vat_cents, total_ttc_cents, deleted_at, created_at, updated_at
            "#,
            quote.id.0,
            quote.title,
            quote.customer_id.0,
            quote.customer_context_id.0,
            quote.status.as_str(),
            quote.deposit_basis,
            quote.deposit_value,
            quote.total_cents,
            quote.total_ht_cents,
            quote.total_vat_cents,
            quote.total_ttc_cents,
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
        updated_at: DateTime<Utc>,
    ) -> Result<Quote, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            QuoteRow,
            r#"
            UPDATE quotes
            SET status = CAST($2 AS text)::quote_status, updated_at = $3
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, reference, title, customer_id, customer_context_id, status::text AS "status!", deposit_basis, deposit_value, total_cents, total_ht_cents, total_vat_cents, total_ttc_cents, deleted_at, created_at, updated_at
            "#,
            id.0,
            status.as_str(),
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

    async fn replace_legal_mention_templates(
        &mut self,
        quote: &Quote,
        template_ids: &[LegalMentionTemplateId],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"DELETE FROM quote_legal_mentions WHERE quote_id = $1"#,
            quote.id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        insert_legal_mention_templates(&mut tx, quote, template_ids).await
    }

    async fn find_legal_mention_template_ids(
        &mut self,
        quote_id: QuoteId,
    ) -> Result<Vec<LegalMentionTemplateId>, CoreError> {
        let mut tx = self.tx.lock().await;
        fetch_legal_mention_template_ids(&mut tx, quote_id).await
    }
}

async fn fetch_legal_mention_template_ids(
    conn: &mut PgConnection,
    quote_id: QuoteId,
) -> Result<Vec<LegalMentionTemplateId>, CoreError> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT template_id
        FROM quote_legal_mentions
        WHERE quote_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
        quote_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    Ok(rows.into_iter().map(LegalMentionTemplateId).collect())
}

async fn insert_legal_mention_templates(
    conn: &mut PgConnection,
    quote: &Quote,
    template_ids: &[LegalMentionTemplateId],
) -> Result<(), CoreError> {
    for template_id in template_ids {
        sqlx::query!(
            r#"
            INSERT INTO quote_legal_mentions (id, org_id, quote_id, template_id)
            VALUES ($1, $2, $3, $4)
            "#,
            generate_uuid_v7(),
            quote.organization_id.0,
            quote.id.0,
            template_id.0,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}

async fn fetch_lines(
    conn: &mut PgConnection,
    quote_id: QuoteId,
) -> Result<Vec<QuoteLine>, CoreError> {
    let rows = sqlx::query_as!(
        QuoteLineRow,
        r#"
        SELECT id, org_id, quote_id, service_rate_id, label, quantity, unit, unit_price_cents, vat_rate, notes, photo_keys, deleted_at, created_at, updated_at
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
                vat_rate,
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
            line.vat_rate,
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
