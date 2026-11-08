use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, OrganizationId};
use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::domain::absence::commands::{
    AbsenceSchedule, CreateAbsenceCommand, PatchAbsenceCommand,
};
use crate::domain::custom_unit::commands::CreateCustomUnitCommand;
use crate::domain::equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand};
use crate::domain::product::commands::{CreateProductCommand, UpdateProductCommand};
use crate::domain::service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand};
use crate::{
    Absence, AbsenceId, AbsenceKind, CustomUnit, Employee, EmployeeCostBasis, EmployeeCostBasisId,
    EmployeeId, Equipment, EquipmentId, MemberId, Product, ProductId, ServiceRate, ServiceRateId,
    SupplierId, salaried_hourly_rate_cents,
};

#[cfg(test)]
#[path = "tests_referentiel.rs"]
mod tests_referentiel;

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.product.list" => list_products(context).await,
        "mestier.product.get" => get_product(context).await,
        "mestier.product.create" => create_product(context).await,
        "mestier.product.update" => update_product(context).await,
        "mestier.product.delete" => delete_product(context).await,
        "mestier.service_rate.list" => list_service_rates(context).await,
        "mestier.service_rate.get" => get_service_rate(context).await,
        "mestier.service_rate.create" => create_service_rate(context).await,
        "mestier.service_rate.update" => update_service_rate(context).await,
        "mestier.service_rate.delete" => delete_service_rate(context).await,
        "mestier.equipment.list" => list_equipment(context).await,
        "mestier.equipment.get" => get_equipment(context).await,
        "mestier.equipment.create" => create_equipment(context).await,
        "mestier.equipment.update" => update_equipment(context).await,
        "mestier.equipment.delete" => delete_equipment(context).await,
        "mestier.custom_unit.list" => list_custom_units(context).await,
        "mestier.custom_unit.create" => create_custom_unit(context).await,
        "mestier.absence.list" => list_absences(context).await,
        "mestier.absence.create" => create_absence(context).await,
        "mestier.absence.update" => update_absence(context).await,
        "mestier.absence.delete" => delete_absence(context).await,
        "mestier.employee_profile.list" => list_employee_profiles(context).await,
        "mestier.employee_profile.upsert" => upsert_employee_profile(context).await,
        "mestier.employee_profile.remove" => remove_employee_profile(context).await,
        "mestier.cost_basis.list" => list_cost_bases(context).await,
        "mestier.cost_basis.set" => set_cost_basis(context).await,
        "mestier.cost_basis.correct" => correct_cost_basis(context).await,
        _ => return None,
    })
}

struct Page {
    limit: u64,
    offset: u64,
}

fn page(context: &ActionContext<'_>) -> Result<Page, String> {
    Ok(Page {
        limit: context
            .optional_whole_number("limit")?
            .unwrap_or(DEFAULT_LIMIT)
            .clamp(1, MAX_LIMIT),
        offset: context.optional_whole_number("offset")?.unwrap_or(0),
    })
}

fn whole(context: &ActionContext<'_>, name: &str) -> Result<i32, String> {
    optional_whole(context, name)?.ok_or_else(|| format!("`{name}` must resolve to a number"))
}

fn optional_whole(context: &ActionContext<'_>, name: &str) -> Result<Option<i32>, String> {
    context
        .optional_whole_number(name)?
        .map(|value| i32::try_from(value).map_err(|_| format!("`{name}` is too large")))
        .transpose()
}

fn strings(context: &ActionContext<'_>, name: &str) -> Result<Vec<String>, String> {
    Ok(context
        .optional_strings(name)?
        .into_iter()
        .map(str::to_owned)
        .collect())
}

fn owned(context: &ActionContext<'_>, name: &str) -> Result<Option<String>, String> {
    Ok(context.optional_text(name)?.map(str::to_owned))
}

fn day(raw: &str, name: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| format!("`{name}` must be a date formatted YYYY-MM-DD"))
}

fn instant(raw: &str, name: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(raw.trim())
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| format!("`{name}` must be an RFC 3339 date and time"))
}

fn ensure_in_run(context: &ActionContext<'_>, owner: OrganizationId) -> Result<(), String> {
    if owner == context.organization_id {
        Ok(())
    } else {
        Err(failure(CoreError::NotFound))
    }
}

fn page_of<T>(items: Vec<T>, page: &Page) -> impl Iterator<Item = T> {
    items
        .into_iter()
        .skip(usize::try_from(page.offset).unwrap_or(usize::MAX))
        .take(usize::try_from(page.limit).unwrap_or(usize::MAX))
}

fn product_json(product: &Product) -> Value {
    json!({
        "id": product.id.0,
        "organization_id": product.organization_id.0,
        "name": product.name,
        "sku": product.sku,
        "unit": product.unit,
        "unit_price_cents": product.unit_price_cents,
        "default_vat_rate_bp": product.default_vat_rate_bp,
        "description": product.description,
        "photo_keys": product.photo_keys,
        "supplier_id": product.supplier_id.map(|id| id.0),
        "manufacturer": product.manufacturer,
        "technical_specs": product.technical_specs,
        "datasheet_keys": product.datasheet_keys,
        "created_at": product.created_at,
        "updated_at": product.updated_at,
    })
}

fn service_rate_json(rate: &ServiceRate) -> Value {
    json!({
        "id": rate.id.0,
        "organization_id": rate.organization_id.0,
        "label": rate.label,
        "unit": rate.unit,
        "rate_cents": rate.rate_cents,
        "default_vat_rate_bp": rate.default_vat_rate_bp,
        "description": rate.description,
        "created_at": rate.created_at,
        "updated_at": rate.updated_at,
    })
}

fn equipment_json(equipment: &Equipment) -> Value {
    json!({
        "id": equipment.id.0,
        "organization_id": equipment.organization_id.0,
        "name": equipment.name,
        "hourly_rate_cents": equipment.hourly_rate_cents,
        "created_at": equipment.created_at,
        "updated_at": equipment.updated_at,
    })
}

fn custom_unit_json(unit: &CustomUnit) -> Value {
    json!({
        "id": unit.id.0,
        "organization_id": unit.organization_id.0,
        "code": unit.code,
        "created_at": unit.created_at,
    })
}

fn absence_json(absence: &Absence) -> Value {
    json!({
        "id": absence.id.0,
        "organization_id": absence.organization_id.0,
        "member_id": absence.member_id.0,
        "kind": absence.kind.as_str(),
        "starts_at": absence.starts_at,
        "ends_at": absence.ends_at,
        "all_day": absence.all_day,
        "note": absence.note,
        "created_at": absence.created_at,
        "updated_at": absence.updated_at,
    })
}

fn effective_rate(
    is_salaried: bool,
    hourly: Option<i32>,
    monthly: Option<i32>,
    weekly_minutes: i32,
) -> Option<i32> {
    if is_salaried {
        salaried_hourly_rate_cents(monthly, weekly_minutes)
    } else {
        hourly
    }
}

fn employee_json(employee: &Employee) -> Value {
    json!({
        "id": employee.id.0,
        "organization_id": employee.organization_id.0,
        "member_id": employee.member_id.0,
        "hourly_rate_cents": employee.hourly_rate_cents,
        "is_salaried": employee.is_salaried,
        "monthly_cost_cents": employee.monthly_cost_cents,
        "effective_hourly_rate_cents": effective_rate(
            employee.is_salaried,
            employee.hourly_rate_cents,
            employee.monthly_cost_cents,
            employee.weekly_contract_minutes,
        ),
        "weekly_contract_minutes": employee.weekly_contract_minutes,
        "created_at": employee.created_at,
        "updated_at": employee.updated_at,
    })
}

fn cost_basis_json(basis: &EmployeeCostBasis) -> Value {
    json!({
        "id": basis.id.0,
        "organization_id": basis.organization_id.0,
        "employee_id": basis.employee_id.0,
        "effective_from": basis.effective_from,
        "effective_to": basis.effective_to,
        "hourly_rate_cents": basis.hourly_rate_cents,
        "is_salaried": basis.is_salaried,
        "monthly_cost_cents": basis.monthly_cost_cents,
        "effective_hourly_rate_cents": effective_rate(
            basis.is_salaried,
            basis.hourly_rate_cents,
            basis.monthly_cost_cents,
            basis.weekly_contract_minutes,
        ),
        "weekly_contract_minutes": basis.weekly_contract_minutes,
        "created_at": basis.created_at,
        "updated_at": basis.updated_at,
    })
}

fn product_id(context: &ActionContext<'_>) -> Result<ProductId, String> {
    context.uuid("product_id").map(ProductId)
}

async fn own_product(context: &ActionContext<'_>, id: ProductId) -> Result<Product, String> {
    let product = context.usecase.get_product(id).await.map_err(failure)?;
    ensure_in_run(context, product.organization_id)?;
    Ok(product)
}

struct ProductFields {
    name: String,
    sku: Option<String>,
    unit: String,
    unit_price_cents: i32,
    default_vat_rate_bp: Option<i32>,
    description: Option<String>,
    photo_keys: Vec<String>,
    supplier_id: Option<SupplierId>,
    manufacturer: Option<String>,
    technical_specs: Option<String>,
    datasheet_keys: Vec<String>,
}

fn product_fields(context: &ActionContext<'_>) -> Result<ProductFields, String> {
    Ok(ProductFields {
        name: context.text("name")?.to_owned(),
        sku: owned(context, "sku")?,
        unit: context.text("unit")?.to_owned(),
        unit_price_cents: whole(context, "unit_price_cents")?,
        default_vat_rate_bp: optional_whole(context, "default_vat_rate_bp")?,
        description: owned(context, "description")?,
        photo_keys: strings(context, "photo_keys")?,
        supplier_id: context.optional_uuid("supplier_id")?.map(SupplierId),
        manufacturer: owned(context, "manufacturer")?,
        technical_specs: owned(context, "technical_specs")?,
        datasheet_keys: strings(context, "datasheet_keys")?,
    })
}

async fn list_products(context: &ActionContext<'_>) -> Result<Value, String> {
    let page = page(context)?;

    context
        .usecase
        .list_products(context.organization_id, page.limit, page.offset)
        .await
        .map(|(items, _)| Value::Array(items.iter().map(product_json).collect()))
        .map_err(failure)
}

async fn get_product(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = product_id(context)?;

    own_product(context, id)
        .await
        .map(|product| product_json(&product))
}

async fn create_product(context: &ActionContext<'_>) -> Result<Value, String> {
    let fields = product_fields(context)?;

    context
        .usecase
        .create_product(CreateProductCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            name: fields.name,
            sku: fields.sku,
            unit: fields.unit,
            unit_price_cents: fields.unit_price_cents,
            default_vat_rate_bp: fields.default_vat_rate_bp,
            description: fields.description,
            photo_keys: fields.photo_keys,
            supplier_id: fields.supplier_id,
            manufacturer: fields.manufacturer,
            technical_specs: fields.technical_specs,
            datasheet_keys: fields.datasheet_keys,
            catalog: None,
        })
        .await
        .map(|product| product_json(&product))
        .map_err(failure)
}

async fn update_product(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = product_id(context)?;
    let fields = product_fields(context)?;
    own_product(context, id).await?;

    context
        .usecase
        .update_product(UpdateProductCommand {
            actor: context.actor(),
            id,
            name: fields.name,
            sku: fields.sku,
            unit: fields.unit,
            unit_price_cents: fields.unit_price_cents,
            default_vat_rate_bp: fields.default_vat_rate_bp,
            description: fields.description,
            photo_keys: fields.photo_keys,
            supplier_id: fields.supplier_id,
            manufacturer: fields.manufacturer,
            technical_specs: fields.technical_specs,
            datasheet_keys: fields.datasheet_keys,
            catalog: None,
        })
        .await
        .map(|product| product_json(&product))
        .map_err(failure)
}

async fn delete_product(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = product_id(context)?;
    own_product(context, id).await?;

    context
        .usecase
        .soft_delete_product(context.actor(), id)
        .await
        .map_err(failure)?;

    Ok(json!({ "product_id": id.0, "deleted": true }))
}

fn service_rate_id(context: &ActionContext<'_>) -> Result<ServiceRateId, String> {
    context.uuid("service_rate_id").map(ServiceRateId)
}

async fn own_service_rate(
    context: &ActionContext<'_>,
    id: ServiceRateId,
) -> Result<ServiceRate, String> {
    let rate = context
        .usecase
        .get_service_rate(id)
        .await
        .map_err(failure)?;
    ensure_in_run(context, rate.organization_id)?;
    Ok(rate)
}

struct RateFields {
    label: String,
    unit: String,
    rate_cents: i32,
    default_vat_rate_bp: Option<i32>,
    description: Option<String>,
}

fn rate_fields(context: &ActionContext<'_>) -> Result<RateFields, String> {
    Ok(RateFields {
        label: context.text("label")?.to_owned(),
        unit: context.text("unit")?.to_owned(),
        rate_cents: whole(context, "rate_cents")?,
        default_vat_rate_bp: optional_whole(context, "default_vat_rate_bp")?,
        description: owned(context, "description")?,
    })
}

async fn list_service_rates(context: &ActionContext<'_>) -> Result<Value, String> {
    let page = page(context)?;

    context
        .usecase
        .list_service_rates(context.organization_id, page.limit, page.offset)
        .await
        .map(|(items, _)| Value::Array(items.iter().map(service_rate_json).collect()))
        .map_err(failure)
}

async fn get_service_rate(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = service_rate_id(context)?;

    own_service_rate(context, id)
        .await
        .map(|rate| service_rate_json(&rate))
}

async fn create_service_rate(context: &ActionContext<'_>) -> Result<Value, String> {
    let fields = rate_fields(context)?;

    context
        .usecase
        .create_service_rate(CreateServiceRateCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            label: fields.label,
            unit: fields.unit,
            rate_cents: fields.rate_cents,
            default_vat_rate_bp: fields.default_vat_rate_bp,
            description: fields.description,
            catalog: None,
        })
        .await
        .map(|rate| service_rate_json(&rate))
        .map_err(failure)
}

async fn update_service_rate(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = service_rate_id(context)?;
    let fields = rate_fields(context)?;
    own_service_rate(context, id).await?;

    context
        .usecase
        .update_service_rate(UpdateServiceRateCommand {
            actor: context.actor(),
            id,
            label: fields.label,
            unit: fields.unit,
            rate_cents: fields.rate_cents,
            default_vat_rate_bp: fields.default_vat_rate_bp,
            description: fields.description,
            catalog: None,
        })
        .await
        .map(|rate| service_rate_json(&rate))
        .map_err(failure)
}

async fn delete_service_rate(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = service_rate_id(context)?;
    own_service_rate(context, id).await?;

    context
        .usecase
        .soft_delete_service_rate(context.actor(), id)
        .await
        .map_err(failure)?;

    Ok(json!({ "service_rate_id": id.0, "deleted": true }))
}

fn equipment_id(context: &ActionContext<'_>) -> Result<EquipmentId, String> {
    context.uuid("equipment_id").map(EquipmentId)
}

async fn own_equipment(context: &ActionContext<'_>, id: EquipmentId) -> Result<Equipment, String> {
    let equipment = context.usecase.get_equipment(id).await.map_err(failure)?;
    ensure_in_run(context, equipment.organization_id)?;
    Ok(equipment)
}

async fn list_equipment(context: &ActionContext<'_>) -> Result<Value, String> {
    let page = page(context)?;

    context
        .usecase
        .list_equipment(context.organization_id, page.limit, page.offset)
        .await
        .map(|(items, _)| Value::Array(items.iter().map(equipment_json).collect()))
        .map_err(failure)
}

async fn get_equipment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = equipment_id(context)?;

    own_equipment(context, id)
        .await
        .map(|equipment| equipment_json(&equipment))
}

async fn create_equipment(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.text("name")?.to_owned();
    let hourly_rate_cents = whole(context, "hourly_rate_cents")?;

    context
        .usecase
        .create_equipment(CreateEquipmentCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            name,
            hourly_rate_cents,
        })
        .await
        .map(|equipment| equipment_json(&equipment))
        .map_err(failure)
}

async fn update_equipment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = equipment_id(context)?;
    let name = context.text("name")?.to_owned();
    let hourly_rate_cents = whole(context, "hourly_rate_cents")?;
    own_equipment(context, id).await?;

    context
        .usecase
        .update_equipment(UpdateEquipmentCommand {
            actor: context.actor(),
            id,
            name,
            hourly_rate_cents,
        })
        .await
        .map(|equipment| equipment_json(&equipment))
        .map_err(failure)
}

async fn delete_equipment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = equipment_id(context)?;
    own_equipment(context, id).await?;

    context
        .usecase
        .soft_delete_equipment(context.actor(), id)
        .await
        .map_err(failure)?;

    Ok(json!({ "equipment_id": id.0, "deleted": true }))
}

async fn list_custom_units(context: &ActionContext<'_>) -> Result<Value, String> {
    let page = page(context)?;

    context
        .usecase
        .list_custom_units(context.organization_id)
        .await
        .map(|units| {
            Value::Array(
                page_of(units, &page)
                    .map(|unit| custom_unit_json(&unit))
                    .collect(),
            )
        })
        .map_err(failure)
}

async fn create_custom_unit(context: &ActionContext<'_>) -> Result<Value, String> {
    let code = context.text("code")?.to_owned();

    context
        .usecase
        .create_custom_unit(CreateCustomUnitCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            code,
        })
        .await
        .map(|unit| custom_unit_json(&unit))
        .map_err(failure)
}

fn absence_id(context: &ActionContext<'_>) -> Result<AbsenceId, String> {
    context.uuid("absence_id").map(AbsenceId)
}

fn absence_kind(raw: &str) -> Result<AbsenceKind, String> {
    raw.parse()
        .map_err(|_| "`kind` must be LEAVE, SICK or UNAVAILABLE".to_owned())
}

async fn own_absence(context: &ActionContext<'_>, id: AbsenceId) -> Result<Absence, String> {
    let absence = context.usecase.get_absence(id).await.map_err(failure)?;
    ensure_in_run(context, absence.organization_id)?;
    Ok(absence)
}

async fn own_member(context: &ActionContext<'_>, id: MemberId) -> Result<(), String> {
    let member = context
        .usecase
        .find_member_seat(id)
        .await
        .map_err(failure)?
        .ok_or_else(|| failure(CoreError::NotFound))?;
    ensure_in_run(context, member.organization_id)
}

async fn list_absences(context: &ActionContext<'_>) -> Result<Value, String> {
    let page = page(context)?;

    context
        .usecase
        .list_absences(context.organization_id, page.limit, page.offset)
        .await
        .map(|(items, _)| Value::Array(items.iter().map(absence_json).collect()))
        .map_err(failure)
}

async fn create_absence(context: &ActionContext<'_>) -> Result<Value, String> {
    let member_id = MemberId(context.uuid("member_id")?);
    let kind = absence_kind(context.text("kind")?)?;
    let starts_at = instant(context.text("starts_at")?, "starts_at")?;
    let ends_at = instant(context.text("ends_at")?, "ends_at")?;
    let all_day = context.optional_bool("all_day")?.unwrap_or(false);
    let note = owned(context, "note")?;
    own_member(context, member_id).await?;

    context
        .usecase
        .create_absence(CreateAbsenceCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            member_id,
            kind: Some(kind),
            type_id: None,
            schedule: AbsenceSchedule::Window {
                starts_at,
                ends_at,
                all_day,
            },
            note,
            proof_file_key: None,
        })
        .await
        .map_err(failure)?
        .map(|absence| absence_json(&absence))
        .map_err(|refusal| refusal.to_string())
}

async fn update_absence(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = absence_id(context)?;
    let mut command = PatchAbsenceCommand::new(id, context.actor());
    command.kind = context
        .optional_text("kind")?
        .map(absence_kind)
        .transpose()?;
    command.starts_at = context
        .optional_text("starts_at")?
        .map(|raw| instant(raw, "starts_at"))
        .transpose()?;
    command.ends_at = context
        .optional_text("ends_at")?
        .map(|raw| instant(raw, "ends_at"))
        .transpose()?;
    command.all_day = context.optional_bool("all_day")?;
    command.note = match (
        context.optional_bool("clear_note")?.unwrap_or(false),
        owned(context, "note")?,
    ) {
        (true, _) => Some(None),
        (false, note) => note.map(Some),
    };
    own_absence(context, id).await?;

    context
        .usecase
        .patch_absence(command)
        .await
        .map(|absence| absence_json(&absence))
        .map_err(failure)
}

async fn delete_absence(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = absence_id(context)?;
    own_absence(context, id).await?;

    context
        .usecase
        .soft_delete_absence(context.actor(), id)
        .await
        .map_err(failure)?;

    Ok(json!({ "absence_id": id.0, "deleted": true }))
}

async fn list_employee_profiles(context: &ActionContext<'_>) -> Result<Value, String> {
    let page = page(context)?;

    context
        .usecase
        .list_employees(
            context.actor(),
            context.organization_id,
            page.limit,
            page.offset,
        )
        .await
        .map(|(items, _)| Value::Array(items.iter().map(employee_json).collect()))
        .map_err(failure)
}

async fn upsert_employee_profile(context: &ActionContext<'_>) -> Result<Value, String> {
    let member_id = MemberId(context.uuid("member_id")?);
    let is_salaried = context
        .optional_bool("is_salaried")?
        .ok_or_else(|| "`is_salaried` must resolve to true or false".to_owned())?;
    let weekly_contract_minutes = whole(context, "weekly_contract_minutes")?;
    let hourly_rate_cents = optional_whole(context, "hourly_rate_cents")?;
    let monthly_cost_cents = optional_whole(context, "monthly_cost_cents")?;
    own_member(context, member_id).await?;

    context
        .usecase
        .upsert_employee_profile(
            context.actor(),
            member_id,
            hourly_rate_cents,
            is_salaried,
            monthly_cost_cents,
            weekly_contract_minutes,
        )
        .await
        .map(|employee| employee_json(&employee))
        .map_err(failure)
}

async fn remove_employee_profile(context: &ActionContext<'_>) -> Result<Value, String> {
    let member_id = MemberId(context.uuid("member_id")?);
    own_member(context, member_id).await?;

    context
        .usecase
        .remove_employee_profile(context.actor(), member_id)
        .await
        .map_err(failure)?;

    Ok(json!({ "member_id": member_id.0, "removed": true }))
}

async fn own_employee(context: &ActionContext<'_>, id: EmployeeId) -> Result<Employee, String> {
    let employee = context.usecase.get_employee(id).await.map_err(failure)?;
    ensure_in_run(context, employee.organization_id)?;
    Ok(employee)
}

async fn list_cost_bases(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = EmployeeId(context.uuid("employee_id")?);
    own_employee(context, id).await?;

    context
        .usecase
        .list_employee_cost_bases(context.actor(), id)
        .await
        .map(|items| Value::Array(items.iter().map(cost_basis_json).collect()))
        .map_err(failure)
}

async fn set_cost_basis(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = EmployeeId(context.uuid("employee_id")?);
    let effective_from = day(context.text("effective_from")?, "effective_from")?;
    let is_salaried = context
        .optional_bool("is_salaried")?
        .ok_or_else(|| "`is_salaried` must resolve to true or false".to_owned())?;
    let weekly_contract_minutes = whole(context, "weekly_contract_minutes")?;
    let hourly_rate_cents = optional_whole(context, "hourly_rate_cents")?;
    let monthly_cost_cents = optional_whole(context, "monthly_cost_cents")?;
    own_employee(context, id).await?;

    context
        .usecase
        .set_employee_cost_basis(
            context.actor(),
            id,
            effective_from,
            is_salaried,
            hourly_rate_cents,
            monthly_cost_cents,
            weekly_contract_minutes,
        )
        .await
        .map(|basis| cost_basis_json(&basis))
        .map_err(failure)
}

async fn correct_cost_basis(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = EmployeeCostBasisId(context.uuid("cost_basis_id")?);
    let effective_from = day(context.text("effective_from")?, "effective_from")?;
    let effective_to = context
        .optional_text("effective_to")?
        .map(|raw| day(raw, "effective_to"))
        .transpose()?;
    let is_salaried = context
        .optional_bool("is_salaried")?
        .ok_or_else(|| "`is_salaried` must resolve to true or false".to_owned())?;
    let weekly_contract_minutes = whole(context, "weekly_contract_minutes")?;
    let hourly_rate_cents = optional_whole(context, "hourly_rate_cents")?;
    let monthly_cost_cents = optional_whole(context, "monthly_cost_cents")?;
    let current = context
        .usecase
        .get_employee_cost_basis(id)
        .await
        .map_err(failure)?;
    ensure_in_run(context, current.organization_id)?;

    context
        .usecase
        .correct_employee_cost_basis(
            context.actor(),
            id,
            effective_from,
            effective_to,
            is_salaried,
            hourly_rate_cents,
            monthly_cost_cents,
            weekly_contract_minutes,
        )
        .await
        .map(|basis| cost_basis_json(&basis))
        .map_err(failure)
}
