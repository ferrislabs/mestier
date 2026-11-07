use rust_decimal::Decimal;
use serde_json::{Value, json};

use super::clients::{Row, array, not_found, optional_array, select, targets_of_run};
use super::{ActionContext, failure};
use crate::domain::project::commands::{CreateProjectFromQuoteCommand, PlannedTaskCommand};
use crate::domain::quote::commands::{
    CreateQuoteCommand, QuoteLineCommand, QuoteTermsCommand, UpdateQuoteCommand,
    UpdateQuoteStatusCommand,
};
use crate::{
    BillingFrequency, CustomerContextId, CustomerId, Project, Quote, QuoteId, QuoteLineId,
    QuoteStatus, ServiceRateId, Task, TaskProposal,
};

#[cfg(test)]
#[path = "tests_devis.rs"]
mod tests_devis;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.quote.get" => get_quote(context).await,
        "mestier.quote.create" => create_quote(context).await,
        "mestier.quote.update" => update_quote(context).await,
        "mestier.quote.update_status" => update_status(context).await,
        "mestier.quote.delete" => delete_quote(context).await,
        "mestier.quote.plan_proposal" => plan_proposal(context).await,
        "mestier.quote.create_plan" => create_plan(context).await,
        _ => return None,
    })
}

fn quote_json(quote: &Quote) -> Value {
    json!({
        "id": quote.id.0,
        "organization_id": quote.organization_id.0,
        "reference": quote.reference,
        "title": quote.title,
        "customer_id": quote.customer_id.0,
        "customer_context_id": quote.customer_context_id.0,
        "status": quote.status.as_str(),
        "net_cents": quote.net_cents,
        "vat_breakdown": quote
            .vat_breakdown
            .iter()
            .map(|line| json!({ "rate_bp": line.rate_bp, "vat_cents": line.vat_cents }))
            .collect::<Vec<_>>(),
        "gross_cents": quote.gross_cents,
        "lines": quote
            .lines
            .iter()
            .map(|line| json!({
                "id": line.id.0,
                "service_rate_id": line.service_rate_id.map(|id| id.0),
                "label": line.label,
                "quantity": line.quantity.normalize().to_string(),
                "unit": line.unit,
                "unit_price_cents": line.unit_price_cents,
                "vat_rate_bp": line.vat_rate_bp,
                "notes": line.notes,
                "photo_keys": line.photo_keys,
                "created_at": line.created_at,
                "updated_at": line.updated_at,
            }))
            .collect::<Vec<_>>(),
        "created_at": quote.created_at,
        "updated_at": quote.updated_at,
    })
}

fn project_json(project: &Project) -> Value {
    json!({
        "id": project.id.0,
        "name": project.name,
        "customer_id": project.customer_id.map(|id| id.0),
        "customer_context_id": project.customer_context_id.map(|id| id.0),
        "quote_id": project.quote_id.map(|id| id.0),
        "created_at": project.created_at,
        "updated_at": project.updated_at,
    })
}

fn task_json(task: &Task) -> Value {
    json!({
        "id": task.id.0,
        "parent_task_id": task.parent_task_id.map(|id| id.0),
        "title": task.title,
        "description": task.description,
        "starts_at": task.starts_at,
        "ends_at": task.ends_at,
        "all_day": task.all_day,
        "project_id": task.project_id.map(|id| id.0),
        "expenses_cents": task.expenses_cents,
        "expenses_label": task.expenses_label,
    })
}

fn proposal_json(proposal: &TaskProposal) -> Value {
    json!({
        "quote_line_id": proposal.quote_line_id.0,
        "title": proposal.title,
        "suggested_minutes": proposal.suggested_minutes,
    })
}

fn quote_id(context: &ActionContext<'_>) -> Result<QuoteId, String> {
    context.uuid("quote_id").map(QuoteId)
}

fn quote_lines(context: &ActionContext<'_>) -> Result<Vec<QuoteLineCommand>, String> {
    array(context, "lines")?
        .iter()
        .enumerate()
        .map(|(position, item)| {
            let row = Row::new(item, "lines", position)?;
            Ok(QuoteLineCommand {
                service_rate_id: row.optional_uuid("service_rate_id")?.map(ServiceRateId),
                product_id: None,
                discount_percent: Decimal::ZERO,
                billing_frequency: BillingFrequency::OneShot,
                label: row.text("label")?.to_owned(),
                quantity: row.decimal("quantity")?,
                unit: row.text("unit")?.to_owned(),
                unit_price_cents: row.integer("unit_price_cents")?,
                vat_rate_bp: row.optional_integer("vat_rate_bp")?,
                notes: row.optional_text("notes")?.map(str::to_owned),
                photo_keys: row.strings("photo_keys")?,
            })
        })
        .collect()
}

fn planned_tasks(context: &ActionContext<'_>) -> Result<Vec<PlannedTaskCommand>, String> {
    optional_array(context, "tasks")?
        .iter()
        .enumerate()
        .map(|(position, item)| {
            let row = Row::new(item, "tasks", position)?;
            let parent_index = row
                .optional_integer("parent_index")?
                .map(|index| {
                    usize::try_from(index).map_err(|_| {
                        "`tasks` element: `parent_index` must not be negative".to_owned()
                    })
                })
                .transpose()?;
            Ok(PlannedTaskCommand {
                parent_index,
                title: row.text("title")?.to_owned(),
                description: row.optional_text("description")?.map(str::to_owned),
                starts_at: row.optional_instant("starts_at")?,
                ends_at: row.optional_instant("ends_at")?,
                all_day: row.bool_or("all_day", false)?,
                blocks_availability: row.bool_or("blocks_availability", false)?,
                expenses_cents: row.optional_integer("expenses_cents")?.unwrap_or(0),
                expenses_label: row.optional_text("expenses_label")?.map(str::to_owned),
                quote_line_ids: row
                    .uuids("quote_line_ids")?
                    .into_iter()
                    .map(QuoteLineId)
                    .collect(),
            })
        })
        .collect()
}

async fn quote_of_run(context: &ActionContext<'_>, id: QuoteId) -> Result<Quote, String> {
    let quote = context.usecase.get_quote(id).await.map_err(failure)?;
    if quote.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(quote)
}

async fn get_quote(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = quote_id(context)?;

    quote_of_run(context, id)
        .await
        .map(|quote| quote_json(&quote))
}

async fn create_quote(context: &ActionContext<'_>) -> Result<Value, String> {
    let title = context.text("title")?.to_owned();
    let customer_id = context.uuid("customer_id").map(CustomerId)?;
    let customer_context_id = context.uuid("customer_context_id").map(CustomerContextId)?;
    let lines = quote_lines(context)?;
    targets_of_run(context, customer_id, customer_context_id).await?;

    context
        .usecase
        .create_quote(
            CreateQuoteCommand {
                organization_id: context.organization_id,
                title,
                customer_id,
                customer_context_id,
                deal_id: None,
                terms: QuoteTermsCommand::default(),
                lines,
            },
            context.actor(),
        )
        .await
        .map_err(failure)?
        .map(|quote| quote_json(&quote))
        .map_err(|refusal| refusal.to_string())
}

async fn update_quote(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = quote_id(context)?;
    let title = context.text("title")?.to_owned();
    let customer_id = context.uuid("customer_id").map(CustomerId)?;
    let customer_context_id = context.uuid("customer_context_id").map(CustomerContextId)?;
    let status = select::<QuoteStatus>(context, "status")?;
    let lines = quote_lines(context)?;
    quote_of_run(context, id).await?;
    targets_of_run(context, customer_id, customer_context_id).await?;

    context
        .usecase
        .update_quote(
            UpdateQuoteCommand {
                id,
                title,
                customer_id,
                customer_context_id,
                deal_id: None,
                status,
                terms: QuoteTermsCommand::default(),
                lines,
            },
            context.actor(),
        )
        .await
        .map_err(failure)?
        .map(|quote| quote_json(&quote))
        .map_err(|refusal| refusal.to_string())
}

async fn update_status(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = quote_id(context)?;
    let status = select::<QuoteStatus>(context, "status")?;
    quote_of_run(context, id).await?;

    context
        .usecase
        .update_quote_status(UpdateQuoteStatusCommand::to(id, status), context.actor())
        .await
        .map_err(failure)?
        .map(|quote| quote_json(&quote))
        .map_err(|refusal| refusal.to_string())
}

async fn delete_quote(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = quote_id(context)?;
    quote_of_run(context, id).await?;

    context
        .usecase
        .soft_delete_quote(id, context.actor())
        .await
        .map_err(failure)?
        .map_err(|refusal| refusal.to_string())?;

    Ok(json!({ "quote_id": id.0, "deleted": true }))
}

async fn plan_proposal(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = quote_id(context)?;
    quote_of_run(context, id).await?;

    context
        .usecase
        .get_quote_plan_proposal(id)
        .await
        .map(|(quote, proposal)| {
            json!({
                "quote": quote_json(&quote),
                "tasks": proposal.iter().map(proposal_json).collect::<Vec<_>>(),
            })
        })
        .map_err(failure)
}

async fn create_plan(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = quote_id(context)?;
    let name = context.text("name")?.to_owned();
    let force_new = context.optional_bool("force_new")?.unwrap_or(false);
    let tasks = planned_tasks(context)?;
    quote_of_run(context, id).await?;

    context
        .usecase
        .create_project_from_quote(CreateProjectFromQuoteCommand {
            actor: context.actor(),
            quote_id: id,
            name,
            force_new,
            tasks,
        })
        .await
        .map(|(project, tasks)| {
            json!({
                "project": project_json(&project),
                "tasks": tasks.iter().map(task_json).collect::<Vec<_>>(),
            })
        })
        .map_err(failure)
}
