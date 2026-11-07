use serde_json::{Value, json};

use super::clients::{
    Row, array, integer, not_found, optional_instant, optional_select, page, select, targets_of_run,
};
use super::{ActionContext, failure};
use crate::domain::invoice::commands::{
    CancelInvoiceCommand, CreateInvoiceCommand, DeleteInvoicePaymentCommand, InvoiceLineCommand,
    IssueCreditNoteCommand, IssueDepositCommand, IssueFinalInvoiceCommand, IssueInvoiceCommand,
    RecordInvoicePaymentCommand, UpdateInvoiceCommand,
};
use crate::{
    CustomerContextId, CustomerId, Invoice, InvoiceId, InvoiceKind, InvoicePayment,
    InvoicePaymentId, InvoiceStatus, OperationNature, OrganizationAddress, ProjectId,
};

#[cfg(test)]
#[path = "tests_factures.rs"]
mod tests_factures;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.invoice.get" => get_invoice(context).await,
        "mestier.invoice.list" => list_invoices(context).await,
        "mestier.invoice.list_by_project" => list_by_project(context).await,
        "mestier.invoice.create" => create_invoice(context).await,
        "mestier.invoice.update" => update_invoice(context).await,
        "mestier.invoice.issue" => issue_invoice(context).await,
        "mestier.invoice.cancel" => cancel_invoice(context).await,
        "mestier.invoice.issue_credit_note" => issue_credit_note(context).await,
        "mestier.invoice.list_credit_notes" => list_credit_notes(context).await,
        "mestier.invoice.balance" => balance(context).await,
        "mestier.invoice.list_payments" => list_payments(context).await,
        "mestier.invoice.outstanding_by_customer" => outstanding(context).await,
        "mestier.invoice.issue_deposit" => issue_deposit(context).await,
        "mestier.invoice.issue_final" => issue_final(context).await,
        "mestier.invoice.project_billing_summary" => billing_summary(context).await,
        "mestier.invoice.record_payment" => record_payment(context).await,
        "mestier.invoice.delete_payment" => delete_payment(context).await,
        _ => return None,
    })
}

fn invoice_json(invoice: &Invoice) -> Value {
    json!({
        "id": invoice.id.0,
        "organization_id": invoice.organization_id.0,
        "number": invoice.number,
        "kind": invoice.kind.as_str(),
        "project_id": invoice.project_id.map(|id| id.0),
        "customer_id": invoice.customer_id.0,
        "customer_context_id": invoice.customer_context_id.0,
        "status": invoice.status.as_str(),
        "issued_at": invoice.issued_at,
        "due_at": invoice.due_at,
        "notes": invoice.notes,
        "operation_nature": invoice.operation_nature.map(OperationNature::as_str),
        "delivery_address": invoice.delivery_address.as_ref().map(|address| json!({
            "line1": address.line1,
            "line2": address.line2,
            "postal_code": address.postal_code,
            "city": address.city,
            "country": address.country,
        })),
        "net_cents": invoice.net_cents,
        "vat_breakdown": invoice
            .vat_breakdown
            .iter()
            .map(|line| json!({ "rate_bp": line.rate_bp, "vat_cents": line.vat_cents }))
            .collect::<Vec<_>>(),
        "gross_cents": invoice.gross_cents,
        "lines": invoice
            .lines
            .iter()
            .map(|line| json!({
                "id": line.id.0,
                "label": line.label,
                "quantity": line.quantity.normalize().to_string(),
                "unit_price_cents": line.unit_price_cents,
                "vat_rate_basis_points": line.vat_rate_basis_points,
                "position": line.position,
                "created_at": line.created_at,
                "updated_at": line.updated_at,
            }))
            .collect::<Vec<_>>(),
        "source_invoice_id": invoice.source_invoice_id.map(|id| id.0),
        "created_at": invoice.created_at,
        "updated_at": invoice.updated_at,
    })
}

fn invoices_json(invoices: &[Invoice]) -> Value {
    Value::Array(invoices.iter().map(invoice_json).collect())
}

fn payment_json(payment: &InvoicePayment) -> Value {
    json!({
        "id": payment.id.0,
        "invoice_id": payment.invoice_id.0,
        "amount_cents": payment.amount_cents,
        "paid_on": payment.paid_on,
        "method": payment.method,
        "reference": payment.reference,
        "note": payment.note,
        "recorded_by": payment.recorded_by.0,
        "created_at": payment.created_at,
        "updated_at": payment.updated_at,
    })
}

fn invoice_id(context: &ActionContext<'_>) -> Result<InvoiceId, String> {
    context.uuid("invoice_id").map(InvoiceId)
}

fn project_id(context: &ActionContext<'_>) -> Result<ProjectId, String> {
    context.uuid("project_id").map(ProjectId)
}

fn invoice_lines(context: &ActionContext<'_>) -> Result<Vec<InvoiceLineCommand>, String> {
    array(context, "lines")?
        .iter()
        .enumerate()
        .map(|(position, item)| {
            let row = Row::new(item, "lines", position)?;
            Ok(InvoiceLineCommand {
                label: row.text("label")?.to_owned(),
                quantity: row.decimal("quantity")?,
                unit_price_cents: row.integer("unit_price_cents")?,
                vat_rate_basis_points: row.optional_integer("vat_rate_basis_points")?,
            })
        })
        .collect()
}

fn delivery_address(context: &ActionContext<'_>) -> Result<Option<OrganizationAddress>, String> {
    let Some(value) = context.optional_object("delivery_address")? else {
        return Ok(None);
    };
    let row = Row::new(&value, "delivery_address", 0)?;
    Ok(Some(OrganizationAddress {
        line1: row.text("line1")?.to_owned(),
        line2: row.optional_text("line2")?.map(str::to_owned),
        postal_code: row.text("postal_code")?.to_owned(),
        city: row.text("city")?.to_owned(),
        country: row.text("country")?.to_owned(),
    }))
}

async fn invoice_of_run(context: &ActionContext<'_>, id: InvoiceId) -> Result<Invoice, String> {
    let invoice = context.usecase.get_invoice(id).await.map_err(failure)?;
    if invoice.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(invoice)
}

async fn project_of_run(context: &ActionContext<'_>, id: ProjectId) -> Result<(), String> {
    let project = context.usecase.get_project(id).await.map_err(failure)?;
    if project.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(())
}

async fn get_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;

    invoice_of_run(context, id)
        .await
        .map(|invoice| invoice_json(&invoice))
}

async fn list_invoices(context: &ActionContext<'_>) -> Result<Value, String> {
    let (limit, offset) = page(context)?;

    context
        .usecase
        .list_invoices(context.organization_id, limit, offset)
        .await
        .map(|(invoices, _)| invoices_json(&invoices))
        .map_err(failure)
}

async fn list_by_project(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    project_of_run(context, id).await?;

    context
        .usecase
        .list_invoices_by_project(id)
        .await
        .map(|invoices| invoices_json(&invoices))
        .map_err(failure)
}

async fn create_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let kind = select::<InvoiceKind>(context, "kind")?;
    let project = context.optional_uuid("project_id")?.map(ProjectId);
    let customer_id = context.uuid("customer_id").map(CustomerId)?;
    let customer_context_id = context.uuid("customer_context_id").map(CustomerContextId)?;
    let due_at = optional_instant(context, "due_at")?;
    let notes = context.optional_text("notes")?.map(str::to_owned);
    let operation_nature = optional_select::<OperationNature>(context, "operation_nature")?;
    let delivery_address = delivery_address(context)?;
    let lines = invoice_lines(context)?;
    targets_of_run(context, customer_id, customer_context_id).await?;
    if let Some(project) = project {
        project_of_run(context, project).await?;
    }

    context
        .usecase
        .create_invoice(CreateInvoiceCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            kind,
            project_id: project,
            customer_id,
            customer_context_id,
            due_at,
            notes,
            operation_nature,
            delivery_address,
            lines,
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn update_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let project = context.optional_uuid("project_id")?.map(ProjectId);
    let customer_id = context.uuid("customer_id").map(CustomerId)?;
    let customer_context_id = context.uuid("customer_context_id").map(CustomerContextId)?;
    let due_at = optional_instant(context, "due_at")?;
    let notes = context.optional_text("notes")?.map(str::to_owned);
    let operation_nature = optional_select::<OperationNature>(context, "operation_nature")?;
    let delivery_address = delivery_address(context)?;
    let lines = invoice_lines(context)?;
    invoice_of_run(context, id).await?;
    targets_of_run(context, customer_id, customer_context_id).await?;
    if let Some(project) = project {
        project_of_run(context, project).await?;
    }

    context
        .usecase
        .update_invoice(UpdateInvoiceCommand {
            id,
            actor: context.actor(),
            project_id: project,
            customer_id,
            customer_context_id,
            due_at,
            notes,
            operation_nature,
            delivery_address,
            lines,
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn issue_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let allow_exceeding_total = context
        .optional_bool("allow_exceeding_total")?
        .unwrap_or(false);
    invoice_of_run(context, id).await?;

    context
        .usecase
        .issue_invoice(IssueInvoiceCommand {
            id,
            actor: context.actor(),
            allow_exceeding_total,
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn cancel_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    invoice_of_run(context, id).await?;

    context
        .usecase
        .cancel_invoice(CancelInvoiceCommand {
            id,
            actor: context.actor(),
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn issue_credit_note(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let lines = invoice_lines(context)?;
    let notes = context.optional_text("notes")?.map(str::to_owned);
    let allow_exceeding_invoice_total = context
        .optional_bool("allow_exceeding_invoice_total")?
        .unwrap_or(false);
    invoice_of_run(context, id).await?;

    context
        .usecase
        .issue_credit_note(IssueCreditNoteCommand {
            source_invoice_id: id,
            actor: context.actor(),
            lines,
            notes,
            allow_exceeding_invoice_total,
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn list_credit_notes(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    invoice_of_run(context, id).await?;

    context
        .usecase
        .get_invoice_credit_notes(id)
        .await
        .map(|credit_notes| invoices_json(&credit_notes))
        .map_err(failure)
}

async fn balance(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let invoice = invoice_of_run(context, id).await?;
    let credit_notes = context
        .usecase
        .get_invoice_credit_notes(id)
        .await
        .map_err(failure)?;
    let payments = context
        .usecase
        .list_invoice_payments(id)
        .await
        .map_err(failure)?;

    let credited_cents: i64 = credit_notes
        .iter()
        .filter(|credit_note| {
            !matches!(
                credit_note.status,
                InvoiceStatus::Draft | InvoiceStatus::Cancelled
            )
        })
        .map(|credit_note| i64::from(credit_note.gross_cents))
        .sum();
    let paid_cents: i64 = payments
        .iter()
        .map(|payment| i64::from(payment.amount_cents))
        .sum();

    Ok(json!({
        "invoice_id": id.0,
        "gross_cents": invoice.gross_cents,
        "credited_cents": credited_cents,
        "paid_cents": paid_cents,
        "remaining_cents": i64::from(invoice.gross_cents) - credited_cents - paid_cents,
    }))
}

async fn list_payments(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    invoice_of_run(context, id).await?;

    context
        .usecase
        .list_invoice_payments(id)
        .await
        .map(|payments| Value::Array(payments.iter().map(payment_json).collect()))
        .map_err(failure)
}

async fn outstanding(context: &ActionContext<'_>) -> Result<Value, String> {
    context
        .usecase
        .outstanding_balance_by_customer(context.organization_id)
        .await
        .map(|balances| {
            Value::Array(
                balances
                    .iter()
                    .map(|balance| {
                        json!({
                            "customer_id": balance.customer_id.0,
                            "outstanding_cents": balance.outstanding_cents,
                            "oldest_due_at": balance.oldest_due_at,
                        })
                    })
                    .collect(),
            )
        })
        .map_err(failure)
}

async fn issue_deposit(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    let percentage_bp = integer(context, "percentage_bp")?;
    let due_at = optional_instant(context, "due_at")?;
    let notes = context.optional_text("notes")?.map(str::to_owned);
    let allow_exceeding_total = context
        .optional_bool("allow_exceeding_total")?
        .unwrap_or(false);
    project_of_run(context, id).await?;

    context
        .usecase
        .issue_deposit(IssueDepositCommand {
            project_id: id,
            actor: context.actor(),
            percentage_bp,
            due_at,
            notes,
            allow_exceeding_total,
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn issue_final(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    let due_at = optional_instant(context, "due_at")?;
    let notes = context.optional_text("notes")?.map(str::to_owned);
    let allow_exceeding_total = context
        .optional_bool("allow_exceeding_total")?
        .unwrap_or(false);
    project_of_run(context, id).await?;

    context
        .usecase
        .issue_final_invoice(IssueFinalInvoiceCommand {
            project_id: id,
            actor: context.actor(),
            due_at,
            notes,
            allow_exceeding_total,
        })
        .await
        .map(|invoice| invoice_json(&invoice))
        .map_err(failure)
}

async fn billing_summary(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    project_of_run(context, id).await?;

    context
        .usecase
        .project_billing_summary(id)
        .await
        .map(|summary| {
            json!({
                "project_id": summary.project_id.0,
                "quoted_cents": summary.quoted_cents,
                "billed_cents": summary.billed_cents,
                "remaining_cents": summary.remaining_cents,
            })
        })
        .map_err(failure)
}

async fn record_payment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let amount_cents = integer(context, "amount_cents")?;
    let paid_on = chrono::NaiveDate::parse_from_str(context.text("paid_on")?.trim(), "%Y-%m-%d")
        .map_err(|_| "`paid_on` must be a date written YYYY-MM-DD".to_owned())?;
    let method = context.text("method")?.to_owned();
    let reference = context.optional_text("reference")?.map(str::to_owned);
    let note = context.optional_text("note")?.map(str::to_owned);
    let allow_exceeding_total = context
        .optional_bool("allow_exceeding_total")?
        .unwrap_or(false);
    invoice_of_run(context, id).await?;
    let recorded_by = context.system_user_id().await?;

    context
        .usecase
        .record_invoice_payment(RecordInvoicePaymentCommand {
            invoice_id: id,
            actor: context.actor(),
            amount_cents,
            paid_on,
            method,
            reference,
            note,
            recorded_by,
            allow_exceeding_total,
        })
        .await
        .map(|payment| payment_json(&payment))
        .map_err(failure)
}

async fn delete_payment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("payment_id").map(InvoicePaymentId)?;
    let payment = context
        .usecase
        .find_payment_by_id(id)
        .await
        .map_err(failure)?
        .filter(|payment| payment.organization_id == context.organization_id)
        .ok_or_else(not_found)?;
    let deleted_by = context.system_user_id().await?;

    context
        .usecase
        .delete_invoice_payment(DeleteInvoicePaymentCommand {
            id: payment.id,
            actor: context.actor(),
            deleted_by,
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "payment_id": payment.id.0, "deleted": true }))
}
