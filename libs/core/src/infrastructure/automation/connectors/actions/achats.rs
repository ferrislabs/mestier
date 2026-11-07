use common::CoreError;
use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::domain::supplier_invoice::commands::{
    ConfirmSupplierInvoiceCommand, CreateSupplierCommand, LineAllocationShare,
    RejectSupplierInvoiceCommand, ReplaceSupplierInvoiceLineAllocationsCommand,
    UpdateSupplierCommand, UpdateSupplierInvoiceNotesCommand,
};
use crate::{
    ProjectId, ProjectSupplierCostLine, Supplier, SupplierId, SupplierInvoice, SupplierInvoiceId,
    SupplierInvoiceLineAllocation, SupplierInvoiceLineId,
};

#[cfg(test)]
mod tests_achats;

const MAX_NAME_CHARS: usize = 200;
const MAX_FIELD_CHARS: usize = 500;
const MAX_NOTES_CHARS: usize = 5_000;
const MAX_ALLOCATIONS: usize = 100;
const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.achats.list_suppliers" => list_suppliers(context).await,
        "mestier.achats.get_supplier" => get_supplier(context).await,
        "mestier.achats.create_supplier" => create_supplier(context).await,
        "mestier.achats.update_supplier" => update_supplier(context).await,
        "mestier.achats.list_supplier_invoices" => list_invoices(context).await,
        "mestier.achats.get_supplier_invoice" => get_invoice(context).await,
        "mestier.achats.update_supplier_invoice_notes" => update_notes(context).await,
        "mestier.achats.confirm_supplier_invoice" => confirm_invoice(context).await,
        "mestier.achats.reject_supplier_invoice" => reject_invoice(context).await,
        "mestier.achats.replace_line_allocations" => replace_allocations(context).await,
        "mestier.achats.get_project_supplier_costs" => project_costs(context).await,
        _ => return None,
    })
}

fn not_found() -> String {
    failure(CoreError::NotFound)
}

fn supplier_json(supplier: &Supplier) -> Value {
    json!({
        "id": supplier.id,
        "organization_id": supplier.organization_id,
        "name": supplier.name,
        "registration_number": supplier.registration_number,
        "vat_number": supplier.vat_number,
        "address": supplier.address,
        "phone": supplier.phone,
        "email": supplier.email,
        "created_at": supplier.created_at,
        "updated_at": supplier.updated_at,
    })
}

fn allocation_json(allocation: &SupplierInvoiceLineAllocation) -> Value {
    json!({
        "id": allocation.id,
        "organization_id": allocation.organization_id,
        "supplier_invoice_line_id": allocation.supplier_invoice_line_id,
        "project_id": allocation.project_id,
        "amount_cents": allocation.amount_cents,
        "created_at": allocation.created_at,
        "updated_at": allocation.updated_at,
    })
}

fn invoice_json(
    invoice: &SupplierInvoice,
    allocations: &[(SupplierInvoiceLineId, Value)],
) -> Value {
    let lines: Vec<Value> = invoice
        .lines
        .iter()
        .map(|line| {
            let own: Vec<&Value> = allocations
                .iter()
                .filter(|(id, _)| *id == line.id)
                .map(|(_, allocation)| allocation)
                .collect();
            json!({
                "id": line.id,
                "supplier_invoice_id": line.supplier_invoice_id,
                "label": line.label,
                "quantity": line.quantity.normalize().to_string(),
                "unit": line.unit,
                "unit_price_cents": line.unit_price_cents,
                "line_total_cents": line.line_total_cents,
                "vat_rate_basis_points": line.vat_rate_basis_points,
                "position": line.position,
                "allocations": own,
                "created_at": line.created_at,
                "updated_at": line.updated_at,
            })
        })
        .collect();
    json!({
        "id": invoice.id,
        "organization_id": invoice.organization_id,
        "supplier_id": invoice.supplier_id,
        "supplier_name": invoice.supplier_name,
        "supplier_registration_number": invoice.supplier_registration_number,
        "supplier_vat_number": invoice.supplier_vat_number,
        "number": invoice.number,
        "issued_on": invoice.issued_on,
        "due_on": invoice.due_on,
        "received_at": invoice.received_at,
        "source": invoice.source,
        "status": invoice.status,
        "currency": invoice.currency,
        "source_file_key": invoice.source_file_key,
        "source_file_mime_type": invoice.source_file_mime_type,
        "notes": invoice.notes,
        "net_cents": invoice.net_cents,
        "vat_breakdown": invoice
            .vat_breakdown
            .iter()
            .map(|line| json!({ "rate_bp": line.rate_bp, "vat_cents": line.vat_cents }))
            .collect::<Vec<_>>(),
        "gross_cents": invoice.gross_cents,
        "lines": lines,
        "created_at": invoice.created_at,
        "updated_at": invoice.updated_at,
    })
}

fn cost_line_json(line: &ProjectSupplierCostLine) -> Value {
    json!({
        "allocation_id": line.allocation_id,
        "supplier_invoice_id": line.supplier_invoice_id,
        "supplier_invoice_number": line.supplier_invoice_number,
        "supplier_name": line.supplier_name,
        "supplier_invoice_line_id": line.supplier_invoice_line_id,
        "line_label": line.line_label,
        "amount_cents": line.amount_cents,
        "created_at": line.created_at,
    })
}

fn page(context: &ActionContext<'_>) -> Result<(u64, u64), String> {
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);
    Ok((limit, offset))
}

fn bounded(context: &ActionContext<'_>, name: &str, max: usize) -> Result<Option<String>, String> {
    let value = context.optional_text(name)?;
    if value.is_some_and(|text| text.chars().count() > max) {
        return Err(format!("`{name}` holds at most {max} characters"));
    }
    Ok(value.map(str::to_owned))
}

struct SupplierInput {
    name: String,
    registration_number: Option<String>,
    vat_number: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
}

fn supplier_input(context: &ActionContext<'_>) -> Result<SupplierInput, String> {
    Ok(SupplierInput {
        name: context.filled("name", MAX_NAME_CHARS)?.to_owned(),
        registration_number: bounded(context, "registration_number", MAX_FIELD_CHARS)?,
        vat_number: bounded(context, "vat_number", MAX_FIELD_CHARS)?,
        address: bounded(context, "address", MAX_FIELD_CHARS)?,
        phone: bounded(context, "phone", MAX_FIELD_CHARS)?,
        email: bounded(context, "email", MAX_FIELD_CHARS)?,
    })
}

async fn supplier_of(context: &ActionContext<'_>, id: SupplierId) -> Result<Supplier, String> {
    let supplier = context.usecase.get_supplier(id).await.map_err(failure)?;
    if supplier.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(supplier)
}

async fn invoice_of(
    context: &ActionContext<'_>,
    id: SupplierInvoiceId,
) -> Result<SupplierInvoice, String> {
    let invoice = context
        .usecase
        .get_supplier_invoice(id)
        .await
        .map_err(failure)?;
    if invoice.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(invoice)
}

fn invoice_id(context: &ActionContext<'_>) -> Result<SupplierInvoiceId, String> {
    context.uuid("supplier_invoice_id").map(SupplierInvoiceId)
}

async fn list_suppliers(context: &ActionContext<'_>) -> Result<Value, String> {
    let (limit, offset) = page(context)?;

    let (suppliers, _) = context
        .usecase
        .list_suppliers(context.organization_id, limit, offset)
        .await
        .map_err(failure)?;

    Ok(Value::Array(suppliers.iter().map(supplier_json).collect()))
}

async fn get_supplier(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = SupplierId(context.uuid("supplier_id")?);

    supplier_of(context, id)
        .await
        .map(|supplier| supplier_json(&supplier))
}

async fn create_supplier(context: &ActionContext<'_>) -> Result<Value, String> {
    let input = supplier_input(context)?;

    context
        .usecase
        .create_supplier(CreateSupplierCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            name: input.name,
            registration_number: input.registration_number,
            vat_number: input.vat_number,
            address: input.address,
            phone: input.phone,
            email: input.email,
        })
        .await
        .map(|supplier| supplier_json(&supplier))
        .map_err(failure)
}

async fn update_supplier(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = SupplierId(context.uuid("supplier_id")?);
    let input = supplier_input(context)?;
    supplier_of(context, id).await?;

    context
        .usecase
        .update_supplier(UpdateSupplierCommand {
            actor: context.actor(),
            id,
            name: input.name,
            registration_number: input.registration_number,
            vat_number: input.vat_number,
            address: input.address,
            phone: input.phone,
            email: input.email,
        })
        .await
        .map(|supplier| supplier_json(&supplier))
        .map_err(failure)
}

async fn list_invoices(context: &ActionContext<'_>) -> Result<Value, String> {
    let (limit, offset) = page(context)?;

    let (invoices, _) = context
        .usecase
        .list_supplier_invoices(context.organization_id, limit, offset)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        invoices
            .iter()
            .map(|invoice| invoice_json(invoice, &[]))
            .collect(),
    ))
}

async fn get_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let invoice = invoice_of(context, id).await?;

    let mut allocations = Vec::new();
    for line in &invoice.lines {
        let own = context
            .usecase
            .list_supplier_invoice_line_allocations(line.id)
            .await
            .map_err(failure)?;
        allocations.extend(own.iter().map(|each| (line.id, allocation_json(each))));
    }

    Ok(invoice_json(&invoice, &allocations))
}

async fn update_notes(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let notes = bounded(context, "notes", MAX_NOTES_CHARS)?;
    invoice_of(context, id).await?;

    context
        .usecase
        .update_supplier_invoice_notes(UpdateSupplierInvoiceNotesCommand { id, notes })
        .await
        .map(|invoice| invoice_json(&invoice, &[]))
        .map_err(failure)
}

async fn confirm_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let notes = bounded(context, "notes", MAX_NOTES_CHARS)?;
    invoice_of(context, id).await?;

    context
        .usecase
        .confirm_supplier_invoice(ConfirmSupplierInvoiceCommand { id, notes })
        .await
        .map(|invoice| invoice_json(&invoice, &[]))
        .map_err(failure)
}

async fn reject_invoice(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = invoice_id(context)?;
    let notes = bounded(context, "notes", MAX_NOTES_CHARS)?;
    invoice_of(context, id).await?;

    context
        .usecase
        .reject_supplier_invoice(RejectSupplierInvoiceCommand { id, notes })
        .await
        .map(|invoice| invoice_json(&invoice, &[]))
        .map_err(failure)
}

fn shares(context: &ActionContext<'_>) -> Result<Vec<LineAllocationShare>, String> {
    let Some(Value::Array(items)) = context.config.get("allocations") else {
        return Err("`allocations` must resolve to an array".to_owned());
    };
    if items.len() > MAX_ALLOCATIONS {
        return Err(format!(
            "`allocations` holds at most {MAX_ALLOCATIONS} items"
        ));
    }
    items
        .iter()
        .enumerate()
        .map(|(position, item)| {
            let project = item
                .get("project_id")
                .and_then(Value::as_str)
                .and_then(|raw| uuid::Uuid::parse_str(raw.trim()).ok())
                .ok_or_else(|| {
                    format!("`allocations` element {position} needs a `project_id` UUID")
                })?;
            let amount = item
                .get("amount_cents")
                .and_then(Value::as_i64)
                .and_then(|amount| i32::try_from(amount).ok())
                .ok_or_else(|| {
                    format!("`allocations` element {position} needs a whole `amount_cents`")
                })?;
            Ok(LineAllocationShare {
                project_id: ProjectId(project),
                amount_cents: amount,
            })
        })
        .collect()
}

async fn replace_allocations(context: &ActionContext<'_>) -> Result<Value, String> {
    let line_id = SupplierInvoiceLineId(context.uuid("supplier_invoice_line_id")?);
    let allocations = shares(context)?;
    let line = context
        .usecase
        .find_supplier_invoice_line(line_id)
        .await
        .map_err(failure)?
        .ok_or_else(not_found)?;
    if line.organization_id != context.organization_id {
        return Err(not_found());
    }
    for share in &allocations {
        let project = context
            .usecase
            .get_project(share.project_id)
            .await
            .map_err(failure)?;
        if project.organization_id != context.organization_id {
            return Err(not_found());
        }
    }

    context
        .usecase
        .replace_supplier_invoice_line_allocations(ReplaceSupplierInvoiceLineAllocationsCommand {
            organization_id: line.organization_id,
            supplier_invoice_id: line.supplier_invoice_id,
            supplier_invoice_line_id: line_id,
            allocations,
        })
        .await
        .map(|saved| Value::Array(saved.iter().map(allocation_json).collect()))
        .map_err(failure)
}

async fn project_costs(context: &ActionContext<'_>) -> Result<Value, String> {
    let project_id = ProjectId(context.uuid("project_id")?);
    let project = context
        .usecase
        .get_project(project_id)
        .await
        .map_err(failure)?;
    if project.organization_id != context.organization_id {
        return Err(not_found());
    }

    let allocated = context
        .usecase
        .allocated_supplier_cost_for_project(project_id)
        .await
        .map_err(failure)?;
    let lines = context
        .usecase
        .project_supplier_cost_lines(project_id)
        .await
        .map_err(failure)?;

    Ok(json!({
        "project_id": project_id,
        "allocated_cents": allocated,
        "lines": lines.iter().map(cost_line_json).collect::<Vec<_>>(),
    }))
}
