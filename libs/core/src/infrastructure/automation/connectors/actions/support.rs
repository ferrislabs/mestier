use std::str::FromStr;

use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::{
    AddEntryCommand, AssignTicketCommand, CreateTicketCommand, CustomFields, DraftCommand,
    EditDraftCommand, GetTicketCommand, ListThreadCommand, ListTicketsCommand, MemberId,
    RemoveBadgeCommand, Ticket, TicketEntry, TicketEntryId, TicketFilter, TicketId, TicketStatus,
    TransitionTicketCommand,
};

const AUTOMATION_AUTHOR: &str = "Automatisation";
const MAX_SUBJECT_CHARS: usize = 200;
const MAX_BODY_CHARS: usize = 20_000;
const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.support.create_ticket" => create_ticket(context).await,
        "mestier.support.get_ticket" => get_ticket(context).await,
        "mestier.support.list_tickets" => list_tickets(context).await,
        "mestier.support.transition_ticket" => transition_ticket(context).await,
        "mestier.support.set_assignee" => set_assignee(context).await,
        "mestier.support.remove_badge" => remove_badge(context).await,
        "mestier.support.list_thread" => list_thread(context).await,
        "mestier.support.send_reply" => send_reply(context).await,
        "mestier.support.save_draft" => save_draft(context).await,
        "mestier.support.edit_draft" => edit_draft(context).await,
        "mestier.support.delete_draft" => delete_draft(context).await,
        "mestier.support.send_draft" => send_draft(context).await,
        _ => return None,
    })
}

fn ticket_json(ticket: &Ticket) -> Value {
    json!({
        "id": ticket.id.0,
        "number": ticket.number.0,
        "subject": ticket.subject,
        "body": ticket.body,
        "status": ticket.status.as_str(),
        "assignee_member_id": ticket.assignee.map(|member| member.0),
        "assigned_at": ticket.assigned_at,
        "custom_fields": ticket.custom_fields.as_map(),
        "badges": ticket
            .badges
            .iter()
            .map(|badge| json!({ "label": badge.label(), "tone": badge.tone().as_str() }))
            .collect::<Vec<_>>(),
        "external_id": ticket.external_id,
        "created_at": ticket.created_at,
        "updated_at": ticket.updated_at,
    })
}

fn entry_json(entry: &TicketEntry) -> Value {
    json!({
        "id": entry.id.0,
        "ticket_id": entry.ticket_id.0,
        "kind": entry.kind.as_str(),
        "author_member_id": entry.author.map(|member| member.0),
        "author_name": entry.author.is_none().then_some(AUTOMATION_AUTHOR),
        "body": entry.body,
        "attachments": entry
            .attachments
            .iter()
            .map(|attachment| json!({
                "id": attachment.id.0,
                "file_name": attachment.file_name,
                "mime_type": attachment.mime_type,
                "size_bytes": attachment.size_bytes,
            }))
            .collect::<Vec<_>>(),
        "created_at": entry.created_at,
        "updated_at": entry.updated_at,
    })
}

fn ticket_id(context: &ActionContext<'_>) -> Result<TicketId, String> {
    context.uuid("ticket_id").map(TicketId)
}

async fn create_ticket(context: &ActionContext<'_>) -> Result<Value, String> {
    let subject = context.filled("subject", MAX_SUBJECT_CHARS)?.to_owned();
    let body = context.filled("body", MAX_BODY_CHARS)?.to_owned();
    let custom_fields = match context.optional_object("custom_fields")? {
        Some(value) => CustomFields::try_from(value).map_err(failure)?,
        None => CustomFields::default(),
    };
    let external_id = context.optional_text("external_id")?.map(str::to_owned);

    let created = context
        .usecase
        .create_support_ticket(CreateTicketCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            subject,
            body,
            custom_fields,
            external_id,
        })
        .await
        .map_err(failure)?;

    let mut output = ticket_json(&created.ticket);
    output["created"] = json!(created.created);
    Ok(output)
}

async fn get_ticket(context: &ActionContext<'_>) -> Result<Value, String> {
    let ticket_id = ticket_id(context)?;

    context
        .usecase
        .get_support_ticket(GetTicketCommand {
            organization_id: context.organization_id,
            ticket_id,
        })
        .await
        .map(|ticket| ticket_json(&ticket))
        .map_err(failure)
}

async fn list_tickets(context: &ActionContext<'_>) -> Result<Value, String> {
    let statuses = context
        .optional_strings("statuses")?
        .into_iter()
        .map(|raw| TicketStatus::from_str(raw).map_err(failure))
        .collect::<Result<Vec<_>, _>>()?;
    let assignee = context.optional_uuid("assignee_member_id")?.map(MemberId);
    let badge = context.optional_text("badge")?.map(str::to_owned);
    let custom_field = match (
        context.optional_text("custom_field_key")?,
        context.optional_text("custom_field_value")?,
    ) {
        (Some(key), Some(value)) => Some((key.to_owned(), value.to_owned())),
        (None, None) => None,
        _ => {
            return Err(
                "`custom_field_key` and `custom_field_value` must be given together".to_owned(),
            );
        }
    };
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);

    let page = context
        .usecase
        .list_support_tickets(ListTicketsCommand {
            organization_id: context.organization_id,
            filter: TicketFilter {
                statuses,
                assignee,
                badge,
                custom_field,
            },
            offset,
            limit,
        })
        .await
        .map_err(failure)?;

    Ok(Value::Array(page.items.iter().map(ticket_json).collect()))
}

async fn transition_ticket(context: &ActionContext<'_>) -> Result<Value, String> {
    let ticket_id = ticket_id(context)?;
    let status = TicketStatus::from_str(context.text("status")?).map_err(failure)?;

    context
        .usecase
        .transition_support_ticket(TransitionTicketCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            ticket_id,
            status,
        })
        .await
        .map(|ticket| ticket_json(&ticket))
        .map_err(failure)
}

async fn set_assignee(context: &ActionContext<'_>) -> Result<Value, String> {
    let ticket_id = ticket_id(context)?;
    let assignee = context.optional_uuid("assignee_member_id")?.map(MemberId);

    context
        .usecase
        .assign_support_ticket(AssignTicketCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            ticket_id,
            assignee,
        })
        .await
        .map(|ticket| ticket_json(&ticket))
        .map_err(failure)
}

async fn remove_badge(context: &ActionContext<'_>) -> Result<Value, String> {
    let ticket_id = ticket_id(context)?;
    let label = context.text("label")?.to_owned();

    context
        .usecase
        .remove_support_ticket_badge(RemoveBadgeCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            ticket_id,
            label: label.clone(),
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "ticket_id": ticket_id.0, "label": label, "removed": true }))
}

async fn list_thread(context: &ActionContext<'_>) -> Result<Value, String> {
    let ticket_id = ticket_id(context)?;
    let client_visible_only = context
        .optional_bool("client_visible_only")?
        .unwrap_or(false);

    context
        .usecase
        .list_support_thread(ListThreadCommand {
            organization_id: context.organization_id,
            ticket_id,
            client_visible_only,
        })
        .await
        .map(|entries| Value::Array(entries.iter().map(entry_json).collect()))
        .map_err(failure)
}

fn entry_id(context: &ActionContext<'_>) -> Result<TicketEntryId, String> {
    context.uuid("entry_id").map(TicketEntryId)
}

fn entry_command(context: &ActionContext<'_>) -> Result<AddEntryCommand, String> {
    Ok(AddEntryCommand {
        actor: context.actor(),
        organization_id: context.organization_id,
        ticket_id: ticket_id(context)?,
        body: context.filled("body", MAX_BODY_CHARS)?.to_owned(),
    })
}

async fn send_reply(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = entry_command(context)?;

    context
        .usecase
        .send_support_automation_reply(command)
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn save_draft(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = entry_command(context)?;

    context
        .usecase
        .save_support_automation_draft(command)
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn edit_draft(context: &ActionContext<'_>) -> Result<Value, String> {
    let entry_id = entry_id(context)?;
    let body = context.filled("body", MAX_BODY_CHARS)?.to_owned();

    context
        .usecase
        .edit_support_automation_draft(EditDraftCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            entry_id,
            body,
        })
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn delete_draft(context: &ActionContext<'_>) -> Result<Value, String> {
    let entry_id = entry_id(context)?;

    context
        .usecase
        .delete_support_automation_draft(DraftCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            entry_id,
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "entry_id": entry_id.0, "deleted": true }))
}

async fn send_draft(context: &ActionContext<'_>) -> Result<Value, String> {
    let entry_id = entry_id(context)?;

    context
        .usecase
        .send_support_automation_draft(DraftCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            entry_id,
        })
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}
