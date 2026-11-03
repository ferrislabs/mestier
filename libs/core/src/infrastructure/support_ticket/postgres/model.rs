use chrono::{DateTime, Utc};
use common::CoreError;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    Badge, CustomFields, MemberId, OrganizationId, Ticket, TicketId, TicketNumber, TicketStatus,
};

#[derive(Debug, Clone)]
pub struct TicketRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub number: i32,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub assignee_member_id: Option<Uuid>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub custom_fields: Value,
    pub badges: Value,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<TicketRow> for Ticket {
    type Error = CoreError;

    fn try_from(row: TicketRow) -> Result<Self, Self::Error> {
        let invalid = |what: &str, error: &dyn std::fmt::Display| {
            CoreError::Internal(format!(
                "stored ticket {} has an invalid {what}: {error}",
                row.id
            ))
        };
        let status = row
            .status
            .parse::<TicketStatus>()
            .map_err(|error| invalid("status", &error))?;
        let custom_fields = CustomFields::try_from(row.custom_fields)
            .map_err(|error| invalid("custom_fields", &error))?;
        let badges = serde_json::from_value::<Vec<Badge>>(row.badges)
            .map_err(|error| invalid("badges", &error))?;

        Ok(Self {
            id: TicketId(row.id),
            organization_id: OrganizationId(row.org_id),
            number: TicketNumber(row.number),
            subject: row.subject,
            body: row.body,
            status,
            assignee: row.assignee_member_id.map(MemberId),
            assigned_at: row.assigned_at,
            custom_fields,
            badges,
            external_id: row.external_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
