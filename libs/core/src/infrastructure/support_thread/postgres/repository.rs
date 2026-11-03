use std::collections::HashMap;

use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    Attachment, AttachmentId, EntryKind, LocatedAttachment, OrganizationId, TicketEntry,
    TicketEntryId, TicketEntryRepository, TicketId,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        support_thread::postgres::model::{AttachmentRow, EntryRow, LocatedAttachmentRow},
    },
};

#[repository(domain = SupportTicketEntry, backend = Postgres)]
pub struct PgSupportTicketEntryRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupportTicketEntryRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

async fn attachments_by_entry(
    connection: &mut PgConnection,
    organization_id: Uuid,
    entry_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Attachment>>, CoreError> {
    let rows = sqlx::query_as!(
        AttachmentRow,
        r#"
        SELECT id, org_id, ticket_id, entry_id, file_key, file_name, mime_type, size_bytes,
               created_at
        FROM support_ticket_attachments
        WHERE org_id = $1 AND entry_id = ANY($2::uuid[])
        ORDER BY created_at, id
        "#,
        organization_id,
        entry_ids,
    )
    .fetch_all(connection)
    .await
    .map_err(map_sqlx_error)?;

    let mut grouped: HashMap<Uuid, Vec<Attachment>> = HashMap::new();
    for row in rows {
        let entry_id = row.entry_id;
        grouped
            .entry(entry_id)
            .or_default()
            .push(Attachment::try_from(row)?);
    }
    Ok(grouped)
}

fn assemble(
    rows: Vec<EntryRow>,
    mut grouped: HashMap<Uuid, Vec<Attachment>>,
) -> Result<Vec<TicketEntry>, CoreError> {
    rows.into_iter()
        .map(|row| {
            let attachments = grouped.remove(&row.id).unwrap_or_default();
            row.into_entry(attachments)
        })
        .collect()
}

impl<'tx> TicketEntryRepository for PgSupportTicketEntryRepository<'tx> {
    async fn create(&mut self, entry: &TicketEntry) -> Result<TicketEntry, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                EntryRow,
                r#"
                INSERT INTO support_ticket_entries (
                    id, org_id, ticket_id, kind, author_member_id, body, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, org_id, ticket_id, kind, author_member_id, body, created_at,
                          updated_at
                "#,
                entry.id.0,
                entry.organization_id.0,
                entry.ticket_id.0,
                entry.kind.as_str(),
                entry.author.map(|id| id.0),
                entry.body,
                entry.created_at,
                entry.updated_at,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        row.into_entry(Vec::new())
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: TicketEntryId,
    ) -> Result<Option<TicketEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EntryRow,
            r#"
            SELECT id, org_id, ticket_id, kind, author_member_id, body, created_at, updated_at
            FROM support_ticket_entries
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let mut grouped = attachments_by_entry(&mut tx, organization_id.0, &[row.id]).await?;
        let attachments = grouped.remove(&row.id).unwrap_or_default();
        drop(tx);

        row.into_entry(attachments).map(Some)
    }

    async fn list_for_ticket(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        client_visible_only: bool,
    ) -> Result<Vec<TicketEntry>, CoreError> {
        let kinds: Vec<&str> = EntryKind::ALL
            .into_iter()
            .filter(|kind| !client_visible_only || kind.is_client_visible())
            .map(EntryKind::as_str)
            .collect();

        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            EntryRow,
            r#"
            SELECT id, org_id, ticket_id, kind, author_member_id, body, created_at, updated_at
            FROM support_ticket_entries
            WHERE org_id = $1 AND ticket_id = $2 AND kind = ANY($3::text[])
            ORDER BY created_at, id
            "#,
            organization_id.0,
            ticket_id.0,
            &kinds as &[&str],
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let entry_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let grouped = attachments_by_entry(&mut tx, organization_id.0, &entry_ids).await?;
        drop(tx);

        assemble(rows, grouped)
    }

    async fn update(&mut self, entry: &TicketEntry) -> Result<TicketEntry, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EntryRow,
            r#"
            UPDATE support_ticket_entries
            SET kind = $3,
                author_member_id = $4,
                body = $5,
                created_at = $6,
                updated_at = $7
            WHERE id = $1 AND org_id = $2
            RETURNING id, org_id, ticket_id, kind, author_member_id, body, created_at,
                      updated_at
            "#,
            entry.id.0,
            entry.organization_id.0,
            entry.kind.as_str(),
            entry.author.map(|id| id.0),
            entry.body,
            entry.created_at,
            entry.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        let mut grouped = attachments_by_entry(&mut tx, entry.organization_id.0, &[row.id]).await?;
        let attachments = grouped.remove(&row.id).unwrap_or_default();
        drop(tx);

        row.into_entry(attachments)
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: TicketEntryId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let deleted = sqlx::query!(
            r#"
            DELETE FROM support_ticket_entries
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if deleted.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn add_attachment(&mut self, attachment: &Attachment) -> Result<Attachment, CoreError> {
        let size_bytes = i64::try_from(attachment.size_bytes)
            .map_err(|_| CoreError::Conflict("attachment size is out of range".to_owned()))?;

        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                AttachmentRow,
                r#"
                INSERT INTO support_ticket_attachments (
                    id, org_id, ticket_id, entry_id, file_key, file_name, mime_type, size_bytes,
                    created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, org_id, ticket_id, entry_id, file_key, file_name, mime_type,
                          size_bytes, created_at
                "#,
                attachment.id.0,
                attachment.organization_id.0,
                attachment.ticket_id.0,
                attachment.entry_id.0,
                attachment.file_key,
                attachment.file_name,
                attachment.mime_type,
                size_bytes,
                attachment.created_at,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        Attachment::try_from(row)
    }

    async fn find_attachment(
        &mut self,
        organization_id: OrganizationId,
        id: AttachmentId,
    ) -> Result<Option<LocatedAttachment>, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                LocatedAttachmentRow,
                r#"
                SELECT attachments.id, attachments.org_id, attachments.ticket_id,
                       attachments.entry_id, attachments.file_key, attachments.file_name,
                       attachments.mime_type, attachments.size_bytes, attachments.created_at,
                       entries.kind AS "entry_kind!"
                FROM support_ticket_attachments attachments
                JOIN support_ticket_entries entries
                  ON entries.id = attachments.entry_id AND entries.org_id = attachments.org_id
                WHERE attachments.id = $1 AND attachments.org_id = $2
                "#,
                id.0,
                organization_id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        row.map(LocatedAttachment::try_from).transpose()
    }
}
