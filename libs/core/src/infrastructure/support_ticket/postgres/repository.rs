use common::CoreError;
use mestier_macros::repository;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    Badge, CustomFields, LastAssignment, MemberId, OrganizationId, Ticket, TicketFilter, TicketId,
    TicketNumber, TicketPage, TicketRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        support_ticket::postgres::model::TicketRow,
    },
};

#[repository(domain = SupportTicket, backend = Postgres)]
pub struct PgSupportTicketRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupportTicketRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> TicketRepository for PgSupportTicketRepository<'tx> {
    async fn create(&mut self, ticket: &Ticket) -> Result<Ticket, CoreError> {
        let inserted = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                TicketRow,
                r#"
                INSERT INTO support_tickets (
                    id, org_id, number, subject, body, status, assignee_member_id, assigned_at,
                    custom_fields, badges, external_id, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (org_id, external_id) WHERE external_id IS NOT NULL DO NOTHING
                RETURNING id, org_id, number, subject, body, status, assignee_member_id,
                          assigned_at, custom_fields, badges, external_id, created_at, updated_at
                "#,
                ticket.id.0,
                ticket.organization_id.0,
                ticket.number.0,
                ticket.subject,
                ticket.body,
                ticket.status.as_str(),
                ticket.assignee.map(|id| id.0),
                ticket.assigned_at,
                Json(&ticket.custom_fields) as Json<&CustomFields>,
                Json(&ticket.badges) as Json<&Vec<Badge>>,
                ticket.external_id,
                ticket.created_at,
                ticket.updated_at,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        if let Some(row) = inserted {
            return Ticket::try_from(row);
        }

        let external_id = ticket.external_id.as_deref().ok_or_else(|| {
            CoreError::Internal("ticket insert returned no row without an external id".to_owned())
        })?;
        self.find_by_external_id(ticket.organization_id, external_id)
            .await?
            .ok_or_else(|| {
                CoreError::Internal(format!(
                    "ticket with external id `{external_id}` conflicted but cannot be read back"
                ))
            })
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: TicketId,
    ) -> Result<Option<Ticket>, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                TicketRow,
                r#"
                SELECT id, org_id, number, subject, body, status, assignee_member_id,
                       assigned_at, custom_fields, badges, external_id, created_at, updated_at
                FROM support_tickets
                WHERE id = $1 AND org_id = $2
                "#,
                id.0,
                organization_id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        row.map(Ticket::try_from).transpose()
    }

    async fn find_by_external_id(
        &mut self,
        organization_id: OrganizationId,
        external_id: &str,
    ) -> Result<Option<Ticket>, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                TicketRow,
                r#"
                SELECT id, org_id, number, subject, body, status, assignee_member_id,
                       assigned_at, custom_fields, badges, external_id, created_at, updated_at
                FROM support_tickets
                WHERE org_id = $1 AND external_id = $2
                "#,
                organization_id.0,
                external_id,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        row.map(Ticket::try_from).transpose()
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &TicketFilter,
        offset: u64,
        limit: u64,
    ) -> Result<TicketPage, CoreError> {
        let statuses: Vec<&str> = filter
            .statuses
            .iter()
            .map(|status| status.as_str())
            .collect();
        let assignee = filter.assignee.map(|id| id.0);
        let badge = filter.badge.as_deref();
        let field_key = filter.custom_field.as_ref().map(|(key, _)| key.as_str());
        let field_value = filter
            .custom_field
            .as_ref()
            .map(|(_, value)| value.as_str());
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);

        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TicketRow,
            r#"
            SELECT id, org_id, number, subject, body, status, assignee_member_id,
                   assigned_at, custom_fields, badges, external_id, created_at, updated_at
            FROM support_tickets
            WHERE org_id = $1
              AND (cardinality($2::text[]) = 0 OR status = ANY($2::text[]))
              AND ($3::uuid IS NULL OR assignee_member_id = $3::uuid)
              AND ($4::text IS NULL
                   OR badges @> jsonb_build_array(jsonb_build_object('label', $4::text)))
              AND ($5::text IS NULL OR custom_fields ->> $5::text = $6::text)
            ORDER BY created_at DESC, number DESC
            OFFSET $7
            LIMIT $8
            "#,
            organization_id.0,
            &statuses as &[&str],
            assignee,
            badge,
            field_key,
            field_value,
            offset,
            limit,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "total!"
            FROM support_tickets
            WHERE org_id = $1
              AND (cardinality($2::text[]) = 0 OR status = ANY($2::text[]))
              AND ($3::uuid IS NULL OR assignee_member_id = $3::uuid)
              AND ($4::text IS NULL
                   OR badges @> jsonb_build_array(jsonb_build_object('label', $4::text)))
              AND ($5::text IS NULL OR custom_fields ->> $5::text = $6::text)
            "#,
            organization_id.0,
            &statuses as &[&str],
            assignee,
            badge,
            field_key,
            field_value,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        drop(tx);

        Ok(TicketPage {
            items: rows
                .into_iter()
                .map(Ticket::try_from)
                .collect::<Result<_, _>>()?,
            total: u64::try_from(total).map_err(|error| {
                CoreError::Internal(format!("ticket count is invalid: {error}"))
            })?,
        })
    }

    async fn update(&mut self, ticket: &Ticket) -> Result<Ticket, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                TicketRow,
                r#"
                UPDATE support_tickets
                SET subject = $3,
                    body = $4,
                    status = $5,
                    assignee_member_id = $6,
                    assigned_at = $7,
                    custom_fields = $8,
                    badges = $9,
                    external_id = $10,
                    updated_at = $11
                WHERE id = $1 AND org_id = $2
                RETURNING id, org_id, number, subject, body, status, assignee_member_id,
                          assigned_at, custom_fields, badges, external_id, created_at, updated_at
                "#,
                ticket.id.0,
                ticket.organization_id.0,
                ticket.subject,
                ticket.body,
                ticket.status.as_str(),
                ticket.assignee.map(|id| id.0),
                ticket.assigned_at,
                Json(&ticket.custom_fields) as Json<&CustomFields>,
                Json(&ticket.badges) as Json<&Vec<Badge>>,
                ticket.external_id,
                ticket.updated_at,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        row.map(Ticket::try_from)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn next_number(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<TicketNumber, CoreError> {
        let mut tx = self.tx.lock().await;
        let number = sqlx::query_scalar!(
            r#"
            INSERT INTO support_ticket_counters (org_id, last_number)
            VALUES ($1, 1)
            ON CONFLICT (org_id) DO UPDATE
            SET last_number = support_ticket_counters.last_number + 1
            RETURNING last_number
            "#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(TicketNumber(number))
    }

    async fn last_assigned_at(
        &mut self,
        organization_id: OrganizationId,
        members: &[MemberId],
    ) -> Result<Vec<LastAssignment>, CoreError> {
        let requested: Vec<Uuid> = members.iter().map(|member| member.0).collect();

        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT requested.member_id AS "member_id!",
                   MAX(tickets.assigned_at) AS "last_assigned_at?"
            FROM unnest($2::uuid[]) WITH ORDINALITY AS requested(member_id, ordinal)
            LEFT JOIN support_tickets tickets
                   ON tickets.assignee_member_id = requested.member_id
                  AND tickets.org_id = $1
            GROUP BY requested.ordinal, requested.member_id
            ORDER BY requested.ordinal
            "#,
            organization_id.0,
            &requested,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| (MemberId(row.member_id), row.last_assigned_at))
            .collect())
    }
}
