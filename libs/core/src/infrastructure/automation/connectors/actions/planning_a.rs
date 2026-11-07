use std::str::FromStr;

use authz::Subject;
use chrono::NaiveDate;
use common::CoreError;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::application::policy::SUBJECT_MEMBER_ID_KEY;
use crate::{ArchiveMaintenanceContractCommand, CustomerContextId, CustomerId};
use crate::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, Cadence, ContractStatus,
    CreateDoeModelCommand, CreateDoeModelFromStarterCommand, CreateMaintenanceContractCommand,
    DateRange, DeleteDoeCommand, Doe, DoeId, DoeModel, DoeModelId, DoeModelSection, DoeParty,
    DoeSection, DoeStyle, DraftSupplyFromProductCommand, DuplicateMaintenanceContractCommand,
    EmployeeRhythm, Intervention, InterventionId, InterventionStatus, IsoWeekNumber,
    MaintenanceContract, MaintenanceContractId, MaintenanceLineId, MaintenanceLineInput, MemberId,
    PlanMaintenanceLinesCommand, ProductId, QuoteId, RecordInterventionCommand,
    RejectInterventionCommand, ReplaceWorkSlotsCommand, ResolveAssignmentReportCommand, Revision,
    RhythmSlotInput, Season, StarterModel, Task, UpdateDoeCommand, UpdateDoeModelCommand,
    UpdateMaintenanceContractCommand, ValidateInterventionCommand, WorkSlot, WorkSlotInput,
};

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;
const MAX_WORK_TIME_DAYS: i64 = 92;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.assignment_report.list" => list_reports(context).await,
        "mestier.assignment_report.resolve" => resolve_report(context).await,
        "mestier.doe_model.list" => list_models(context).await,
        "mestier.doe_model.get" => get_model(context).await,
        "mestier.doe_model.create" => create_model(context).await,
        "mestier.doe_model.create_from_starter" => create_from_starter(context).await,
        "mestier.doe_model.update" => update_model(context).await,
        "mestier.doe_model.archive" => archive_model(context).await,
        "mestier.doe_model.restore" => restore_model(context).await,
        "mestier.doe.list" => list_does(context).await,
        "mestier.doe.get" => get_doe(context).await,
        "mestier.doe.update" => update_doe(context).await,
        "mestier.doe.delete" => delete_doe(context).await,
        "mestier.doe.draft_supply" => draft_supply(context).await,
        "mestier.maintenance_contract.list" => list_contracts(context).await,
        "mestier.maintenance_contract.get" => get_contract(context).await,
        "mestier.maintenance_contract.create" => create_contract(context).await,
        "mestier.maintenance_contract.update" => update_contract(context).await,
        "mestier.maintenance_contract.archive" => archive_contract(context).await,
        "mestier.maintenance_contract.duplicate" => duplicate_contract(context).await,
        "mestier.maintenance.get_grid" => get_grid(context).await,
        "mestier.maintenance.get_recap" => get_recap(context).await,
        "mestier.maintenance.plan_lines" => plan_lines(context).await,
        "mestier.maintenance_intervention.list" => list_interventions(context).await,
        "mestier.maintenance_intervention.record" => record_intervention(context).await,
        "mestier.maintenance_intervention.validate" => validate_intervention(context).await,
        "mestier.maintenance_intervention.reject" => reject_intervention(context).await,
        "mestier.work_time.get" => get_work_time(context).await,
        "mestier.work_time.put_rhythm" => put_rhythm(context).await,
        "mestier.work_time.put_work_slots" => put_work_slots(context).await,
        _ => return None,
    })
}

fn not_found() -> String {
    failure(CoreError::NotFound)
}

fn page(context: &ActionContext<'_>) -> Result<(u64, u64), String> {
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);
    Ok((limit, offset))
}

fn whole(context: &ActionContext<'_>, name: &str) -> Result<u64, String> {
    context
        .optional_whole_number(name)?
        .ok_or_else(|| format!("`{name}` must be a whole number"))
}

fn year(context: &ActionContext<'_>, name: &str) -> Result<i32, String> {
    i32::try_from(whole(context, name)?).map_err(|_| format!("`{name}` must be a year"))
}

fn json_field<T: DeserializeOwned>(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<Option<T>, String> {
    match context.config.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => T::deserialize(value)
            .map(Some)
            .map_err(|_| format!("`{name}` does not have the expected shape")),
    }
}

fn required_json<T: DeserializeOwned>(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<T, String> {
    json_field(context, name)?.ok_or_else(|| format!("`{name}` must be provided"))
}

fn parse_date(raw: &str, name: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map_err(|_| format!("`{name}` must be a date like 2026-10-12"))
}

fn date(context: &ActionContext<'_>, name: &str) -> Result<NaiveDate, String> {
    parse_date(context.text(name)?, name)
}

fn optional_date(context: &ActionContext<'_>, name: &str) -> Result<Option<NaiveDate>, String> {
    context
        .optional_text(name)?
        .map(|raw| parse_date(raw, name))
        .transpose()
}

fn parse_week(raw: &str) -> Result<(i32, IsoWeekNumber), String> {
    let invalid = || "`week` must be an ISO week like 2026-W43".to_owned();
    let (year, week) = raw.split_once("-W").ok_or_else(invalid)?;
    let year: i32 = year.parse().map_err(|_| invalid())?;
    let week: u8 = week.parse().map_err(|_| invalid())?;
    let week = IsoWeekNumber::try_from(week).map_err(|_| invalid())?;
    Ok((year, week))
}

fn acting_member(member: MemberId) -> Subject {
    Subject::system().with_property(SUBJECT_MEMBER_ID_KEY, member.0.to_string())
}

async fn member_of_organization(
    context: &ActionContext<'_>,
    member: MemberId,
) -> Result<MemberId, String> {
    match context
        .usecase
        .find_member_seat(member)
        .await
        .map_err(failure)?
    {
        Some(seat) if seat.organization_id == context.organization_id => Ok(member),
        _ => Err(not_found()),
    }
}

fn report_json(report: &AssignmentReport) -> Value {
    json!({
        "id": report.id,
        "organization_id": report.organization_id,
        "task_assignment_id": report.task_assignment_id,
        "reported_minutes": report.reported_minutes,
        "comment": report.comment,
        "reported_by": report.reported_by,
        "resolution": report.resolution,
        "resolved_by": report.resolved_by,
        "resolved_at": report.resolved_at,
        "resolution_note": report.resolution_note,
        "created_at": report.created_at,
        "updated_at": report.updated_at,
    })
}

async fn list_reports(context: &ActionContext<'_>) -> Result<Value, String> {
    let resolution = match context.optional_text("resolution")? {
        Some(raw) => AssignmentReportResolution::from_str(raw)
            .map_err(|_| "`resolution` must be PENDING, APPLIED or DISMISSED".to_owned())?,
        None => AssignmentReportResolution::Pending,
    };
    let (limit, offset) = page(context)?;

    let (reports, _) = context
        .usecase
        .list_assignment_reports_by_organization(
            context.organization_id,
            Some(resolution),
            limit,
            offset,
        )
        .await
        .map_err(failure)?;

    Ok(Value::Array(reports.iter().map(report_json).collect()))
}

async fn resolve_report(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = AssignmentReportId(context.uuid("assignment_report_id")?);
    let resolution = match AssignmentReportResolution::from_str(context.text("resolution")?) {
        Ok(AssignmentReportResolution::Pending) | Err(_) => {
            return Err("`resolution` must be APPLIED or DISMISSED".to_owned());
        }
        Ok(resolution) => resolution,
    };
    let resolution_note = context.optional_text("resolution_note")?.map(str::to_owned);
    let resolved_by = MemberId(context.uuid("resolved_by_member_id")?);

    let report = context
        .usecase
        .get_assignment_report(id)
        .await
        .map_err(failure)?;
    if report.organization_id != context.organization_id {
        return Err(not_found());
    }
    let resolved_by = member_of_organization(context, resolved_by).await?;

    context
        .usecase
        .resolve_assignment_report(ResolveAssignmentReportCommand {
            id,
            resolved_by,
            resolution,
            resolution_note,
        })
        .await
        .map(|report| report_json(&report))
        .map_err(failure)
}

fn model_json(model: &DoeModel) -> Value {
    json!({
        "id": model.id,
        "organization_id": model.organization_id,
        "name": model.name,
        "description": model.description,
        "sections": model.sections,
        "style": model.style,
        "archived_at": model.archived_at,
        "created_at": model.created_at,
        "updated_at": model.updated_at,
    })
}

async fn list_models(context: &ActionContext<'_>) -> Result<Value, String> {
    let include_archived = context.optional_bool("include_archived")?.unwrap_or(false);
    let (limit, offset) = page(context)?;

    let (models, _) = context
        .usecase
        .list_doe_models(context.organization_id, include_archived, limit, offset)
        .await
        .map_err(failure)?;

    Ok(Value::Array(models.iter().map(model_json).collect()))
}

async fn get_model(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeModelId(context.uuid("doe_model_id")?);

    model_of_organization(context, id)
        .await
        .map(|model| model_json(&model))
}

async fn model_of_organization(
    context: &ActionContext<'_>,
    id: DoeModelId,
) -> Result<DoeModel, String> {
    let model = context.usecase.get_doe_model(id).await.map_err(failure)?;
    if model.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(model)
}

async fn create_model(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.text("name")?.to_owned();
    let description = context.optional_text("description")?.map(str::to_owned);
    let sections: Vec<DoeModelSection> = required_json(context, "sections")?;
    let style: Option<DoeStyle> = json_field(context, "style")?;

    context
        .usecase
        .create_doe_model(CreateDoeModelCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            name,
            description,
            sections,
            style,
        })
        .await
        .map(|model| model_json(&model))
        .map_err(failure)
}

async fn create_from_starter(context: &ActionContext<'_>) -> Result<Value, String> {
    let starter = StarterModel::deserialize(Value::String(context.text("starter")?.to_owned()))
        .map_err(|_| "`starter` is not a known starter model".to_owned())?;

    context
        .usecase
        .create_doe_model_from_starter(CreateDoeModelFromStarterCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            starter,
        })
        .await
        .map(|model| model_json(&model))
        .map_err(failure)
}

async fn update_model(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeModelId(context.uuid("doe_model_id")?);
    let name = context.text("name")?.to_owned();
    let description = context.optional_text("description")?.map(str::to_owned);
    let sections: Vec<DoeModelSection> = required_json(context, "sections")?;
    let style: Option<DoeStyle> = json_field(context, "style")?;

    model_of_organization(context, id).await?;

    context
        .usecase
        .update_doe_model(UpdateDoeModelCommand {
            actor: context.actor(),
            id,
            name,
            description,
            sections,
            style,
        })
        .await
        .map(|model| model_json(&model))
        .map_err(failure)
}

async fn archive_model(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeModelId(context.uuid("doe_model_id")?);
    model_of_organization(context, id).await?;

    context
        .usecase
        .archive_doe_model(context.actor(), id)
        .await
        .map_err(failure)?;

    model_of_organization(context, id)
        .await
        .map(|model| model_json(&model))
}

async fn restore_model(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeModelId(context.uuid("doe_model_id")?);
    model_of_organization(context, id).await?;

    context
        .usecase
        .restore_doe_model(context.actor(), id)
        .await
        .map_err(failure)?;

    model_of_organization(context, id)
        .await
        .map(|model| model_json(&model))
}

fn doe_json(doe: &Doe) -> Value {
    json!({
        "id": doe.id,
        "project_id": doe.project_id,
        "model_id": doe.model_id,
        "title": doe.title,
        "place": doe.place,
        "lot": doe.lot,
        "issued_on": doe.issued_on,
        "author": doe.author,
        "revision": doe.revision,
        "cover_photo_key": doe.cover_photo_key,
        "parties": doe.parties,
        "sections": doe.sections,
        "created_at": doe.created_at,
        "updated_at": doe.updated_at,
    })
}

async fn doe_of_organization(context: &ActionContext<'_>, id: DoeId) -> Result<Doe, String> {
    let doe = context.usecase.get_doe(id).await.map_err(failure)?;
    if doe.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(doe)
}

async fn list_does(context: &ActionContext<'_>) -> Result<Value, String> {
    let (limit, offset) = page(context)?;

    let (does, _) = context
        .usecase
        .list_does(context.organization_id, limit, offset)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        does.iter()
            .map(|summary| {
                json!({
                    "id": summary.id,
                    "project_id": summary.project_id,
                    "project_name": summary.project_name,
                    "title": summary.title,
                    "revision": summary.revision,
                    "issued_on": summary.issued_on,
                    "updated_at": summary.updated_at,
                })
            })
            .collect(),
    ))
}

async fn get_doe(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeId(context.uuid("doe_id")?);

    doe_of_organization(context, id)
        .await
        .map(|doe| doe_json(&doe))
}

async fn update_doe(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeId(context.uuid("doe_id")?);
    let title = context.text("title")?.to_owned();
    let place = context.optional_text("place")?.map(str::to_owned);
    let lot = context.optional_text("lot")?.map(str::to_owned);
    let issued_on = date(context, "issued_on")?;
    let author = context.optional_text("author")?.map(str::to_owned);
    let revision = Revision::try_from(context.text("revision")?.to_owned()).map_err(failure)?;
    let cover_photo_key = context.optional_text("cover_photo_key")?.map(str::to_owned);
    let parties: Vec<DoeParty> = required_json(context, "parties")?;
    let sections: Vec<DoeSection> = required_json(context, "sections")?;
    let expected_updated_at = context
        .optional_text("expected_updated_at")?
        .map(|raw| {
            chrono::DateTime::parse_from_rfc3339(raw)
                .map(|instant| instant.to_utc())
                .map_err(|_| "`expected_updated_at` must be an RFC 3339 instant".to_owned())
        })
        .transpose()?;

    doe_of_organization(context, id).await?;

    context
        .usecase
        .update_doe(UpdateDoeCommand {
            actor: context.actor(),
            id,
            title,
            place,
            lot,
            issued_on,
            author,
            revision,
            cover_photo_key,
            parties,
            sections,
            expected_updated_at,
        })
        .await
        .map(|doe| doe_json(&doe))
        .map_err(failure)
}

async fn delete_doe(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = DoeId(context.uuid("doe_id")?);
    doe_of_organization(context, id).await?;

    context
        .usecase
        .delete_doe(DeleteDoeCommand {
            actor: context.actor(),
            id,
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "doe_id": id, "deleted": true }))
}

async fn draft_supply(context: &ActionContext<'_>) -> Result<Value, String> {
    let doe_id = DoeId(context.uuid("doe_id")?);
    let product_id = ProductId(context.uuid("product_id")?);
    doe_of_organization(context, doe_id).await?;

    context
        .usecase
        .draft_doe_supply(DraftSupplyFromProductCommand {
            actor: context.actor(),
            doe_id,
            product_id,
        })
        .await
        .map(|supply| json!(supply))
        .map_err(failure)
}

fn contract_json(contract: &MaintenanceContract) -> Value {
    json!({
        "id": contract.id,
        "customer_id": contract.customer_id,
        "customer_context_id": contract.customer_context_id,
        "project_id": contract.project_id,
        "reference": contract.reference,
        "season": contract.season,
        "manager_member_id": contract.manager_member_id,
        "amount_cents": contract.amount_cents,
        "quote_id": contract.quote_id,
        "notes": contract.notes,
        "status": contract.status,
        "lines": contract.lines,
        "archived_at": contract.archived_at,
        "created_at": contract.created_at,
        "updated_at": contract.updated_at,
    })
}

#[derive(Deserialize)]
struct LineRaw {
    #[serde(default)]
    id: Option<MaintenanceLineId>,
    label: String,
    cadence: Cadence,
    #[serde(default)]
    minutes_per_intervention: Option<u32>,
}

struct ContractFields {
    reference: Option<String>,
    season: Season,
    manager_member_id: Option<MemberId>,
    amount_cents: Option<i64>,
    quote_id: Option<QuoteId>,
    notes: Option<String>,
    status: ContractStatus,
    lines: Vec<MaintenanceLineInput>,
}

fn contract_fields(context: &ActionContext<'_>) -> Result<ContractFields, String> {
    let status = match context.optional_text("status")? {
        Some(raw) => ContractStatus::from_str(raw)
            .map_err(|_| "`status` must be ACTIVE or DRAFT".to_owned())?,
        None => ContractStatus::Active,
    };
    let amount_cents = context
        .optional_whole_number("amount_cents")?
        .map(|amount| i64::try_from(amount).map_err(|_| "`amount_cents` is too large".to_owned()))
        .transpose()?;
    let lines: Vec<LineRaw> = required_json(context, "lines")?;

    Ok(ContractFields {
        reference: context.optional_text("reference")?.map(str::to_owned),
        season: required_json(context, "season")?,
        manager_member_id: context.optional_uuid("manager_member_id")?.map(MemberId),
        amount_cents,
        quote_id: context.optional_uuid("quote_id")?.map(QuoteId),
        notes: context.optional_text("notes")?.map(str::to_owned),
        status,
        lines: lines
            .into_iter()
            .map(|line| MaintenanceLineInput {
                id: line.id,
                label: line.label,
                cadence: line.cadence,
                minutes_per_intervention: line.minutes_per_intervention,
            })
            .collect(),
    })
}

async fn contract_of_organization(
    context: &ActionContext<'_>,
    id: MaintenanceContractId,
) -> Result<MaintenanceContract, String> {
    context
        .usecase
        .get_maintenance_contract(context.actor(), context.organization_id, id)
        .await
        .map_err(failure)
}

async fn list_contracts(context: &ActionContext<'_>) -> Result<Value, String> {
    let season = year(context, "season")?;
    let include_archived = context.optional_bool("include_archived")?.unwrap_or(false);

    context
        .usecase
        .list_maintenance_contracts(
            context.actor(),
            context.organization_id,
            season,
            include_archived,
        )
        .await
        .map(|contracts| Value::Array(contracts.iter().map(contract_json).collect()))
        .map_err(failure)
}

async fn get_contract(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = MaintenanceContractId(context.uuid("contract_id")?);

    contract_of_organization(context, id)
        .await
        .map(|contract| contract_json(&contract))
}

async fn create_contract(context: &ActionContext<'_>) -> Result<Value, String> {
    let customer_id = CustomerId(context.uuid("customer_id")?);
    let customer_context_id = CustomerContextId(context.uuid("customer_context_id")?);
    let fields = contract_fields(context)?;

    context
        .usecase
        .create_maintenance_contract(CreateMaintenanceContractCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            customer_id,
            customer_context_id,
            reference: fields.reference,
            season: fields.season,
            manager_member_id: fields.manager_member_id,
            amount_cents: fields.amount_cents,
            quote_id: fields.quote_id,
            notes: fields.notes,
            status: fields.status,
            lines: fields.lines,
        })
        .await
        .map(|contract| contract_json(&contract))
        .map_err(failure)
}

async fn update_contract(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = MaintenanceContractId(context.uuid("contract_id")?);
    let fields = contract_fields(context)?;
    contract_of_organization(context, id).await?;

    context
        .usecase
        .update_maintenance_contract(UpdateMaintenanceContractCommand {
            actor: context.actor(),
            id,
            reference: fields.reference,
            season: fields.season,
            manager_member_id: fields.manager_member_id,
            amount_cents: fields.amount_cents,
            quote_id: fields.quote_id,
            notes: fields.notes,
            status: fields.status,
            lines: fields.lines,
        })
        .await
        .map(|contract| contract_json(&contract))
        .map_err(failure)
}

async fn archive_contract(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = MaintenanceContractId(context.uuid("contract_id")?);
    contract_of_organization(context, id).await?;

    context
        .usecase
        .archive_maintenance_contract(ArchiveMaintenanceContractCommand {
            actor: context.actor(),
            id,
        })
        .await
        .map(|contract| contract_json(&contract))
        .map_err(failure)
}

async fn duplicate_contract(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = MaintenanceContractId(context.uuid("contract_id")?);
    contract_of_organization(context, id).await?;

    context
        .usecase
        .duplicate_maintenance_contract(DuplicateMaintenanceContractCommand {
            actor: context.actor(),
            id,
        })
        .await
        .map(|contract| contract_json(&contract))
        .map_err(failure)
}

async fn get_grid(context: &ActionContext<'_>) -> Result<Value, String> {
    let season = year(context, "season")?;

    context
        .usecase
        .maintenance_grid(context.actor(), context.organization_id, season)
        .await
        .map(|grid| json!(grid))
        .map_err(failure)
}

async fn get_recap(context: &ActionContext<'_>) -> Result<Value, String> {
    let (year, week) = parse_week(context.text("week")?)?;

    context
        .usecase
        .maintenance_recap(context.actor(), context.organization_id, year, week)
        .await
        .map(|recap| json!(recap))
        .map_err(failure)
}

fn task_json(task: &Task) -> Value {
    json!({
        "id": task.id,
        "organization_id": task.organization_id,
        "parent_task_id": task.parent_task_id,
        "title": task.title,
        "description": task.description,
        "starts_at": task.starts_at,
        "ends_at": task.ends_at,
        "all_day": task.all_day,
        "status": task.status,
        "blocks_availability": task.blocks_availability,
        "customer_id": task.customer_id,
        "customer_context_id": task.customer_context_id,
        "quote_id": task.quote_id,
        "project_id": task.project_id,
        "recurrence_id": task.recurrence_id,
        "maintenance_line_id": task.maintenance_line_id,
        "expenses_cents": task.expenses_cents,
        "expenses_label": task.expenses_label,
        "member_ids": task
            .assignments
            .iter()
            .map(|assignment| assignment.member_id)
            .collect::<Vec<_>>(),
        "assignments": task
            .assignments
            .iter()
            .map(|assignment| json!({ "id": assignment.id, "member_id": assignment.member_id }))
            .collect::<Vec<_>>(),
        "created_at": task.created_at,
        "updated_at": task.updated_at,
    })
}

async fn plan_lines(context: &ActionContext<'_>) -> Result<Value, String> {
    let (year, week) = parse_week(context.text("week")?)?;
    let line_ids: Vec<MaintenanceLineId> = required_json(context, "line_ids")?;

    context
        .usecase
        .plan_maintenance_lines(PlanMaintenanceLinesCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            line_ids,
            year,
            week,
        })
        .await
        .map(|tasks| Value::Array(tasks.iter().map(task_json).collect()))
        .map_err(failure)
}

fn intervention_json(intervention: &Intervention) -> Value {
    json!({
        "id": intervention.id,
        "line_id": intervention.line_id,
        "done_on": intervention.done_on,
        "status": intervention.status,
        "task_id": intervention.task_id,
        "proposed_by": intervention.proposed_by,
        "resolved_by": intervention.resolved_by,
        "resolved_at": intervention.resolved_at,
        "note": intervention.note,
        "created_at": intervention.created_at,
    })
}

async fn list_interventions(context: &ActionContext<'_>) -> Result<Value, String> {
    let status = match context.optional_text("status")? {
        Some(raw) => InterventionStatus::from_str(raw)
            .map_err(|_| "`status` must be PROPOSED, VALIDATED or REJECTED".to_owned())?,
        None => InterventionStatus::Proposed,
    };
    let (limit, offset) = page(context)?;

    let (interventions, _) = context
        .usecase
        .list_maintenance_interventions(
            context.actor(),
            context.organization_id,
            status,
            limit,
            offset,
        )
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        interventions.iter().map(intervention_json).collect(),
    ))
}

async fn record_intervention(context: &ActionContext<'_>) -> Result<Value, String> {
    let line_id = MaintenanceLineId(context.uuid("line_id")?);
    let done_on = date(context, "done_on")?;
    let member = MemberId(context.uuid("member_id")?);
    let member = member_of_organization(context, member).await?;

    context
        .usecase
        .record_maintenance_intervention(
            context.organization_id,
            RecordInterventionCommand {
                actor: acting_member(member),
                line_id,
                done_on,
            },
        )
        .await
        .map(|intervention| intervention_json(&intervention))
        .map_err(failure)
}

async fn validate_intervention(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = InterventionId(context.uuid("intervention_id")?);
    let member = MemberId(context.uuid("member_id")?);
    let member = member_of_organization(context, member).await?;

    context
        .usecase
        .validate_maintenance_intervention(
            context.organization_id,
            ValidateInterventionCommand {
                actor: acting_member(member),
                id,
            },
        )
        .await
        .map(|intervention| intervention_json(&intervention))
        .map_err(failure)
}

async fn reject_intervention(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = InterventionId(context.uuid("intervention_id")?);
    let member = MemberId(context.uuid("member_id")?);
    let note = context.optional_text("note")?.map(str::to_owned);
    let member = member_of_organization(context, member).await?;

    context
        .usecase
        .reject_maintenance_intervention(
            context.organization_id,
            RejectInterventionCommand {
                actor: acting_member(member),
                id,
                note,
            },
        )
        .await
        .map(|intervention| intervention_json(&intervention))
        .map_err(failure)
}

fn rhythm_json(rhythm: &EmployeeRhythm) -> Value {
    json!({
        "id": rhythm.id,
        "organization_id": rhythm.organization_id,
        "employee_id": rhythm.employee_id,
        "effective_from": rhythm.effective_from,
        "effective_to": rhythm.effective_to,
        "slots": rhythm
            .slots
            .iter()
            .map(|slot| json!({
                "weekday": slot.weekday,
                "starts_minute": slot.starts_minute,
                "ends_minute": slot.ends_minute,
            }))
            .collect::<Vec<_>>(),
        "created_at": rhythm.created_at,
        "updated_at": rhythm.updated_at,
    })
}

fn work_slot_json(slot: &WorkSlot) -> Value {
    json!({
        "id": slot.id,
        "organization_id": slot.organization_id,
        "member_id": slot.member_id,
        "work_date": slot.work_date,
        "starts_minute": slot.starts_minute,
        "ends_minute": slot.ends_minute,
    })
}

#[derive(Deserialize)]
struct RhythmSlotRaw {
    weekday: i16,
    starts_minute: i16,
    ends_minute: i16,
}

#[derive(Deserialize)]
struct WorkSlotRaw {
    work_date: NaiveDate,
    starts_minute: i16,
    ends_minute: i16,
}

async fn get_work_time(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = MemberId(context.uuid("member_id")?);
    let from = date(context, "from")?;
    let to = date(context, "to")?;
    let range = DateRange::new(from, to).map_err(failure)?;
    if range.span_days() > MAX_WORK_TIME_DAYS {
        return Err(format!(
            "the window must not exceed {MAX_WORK_TIME_DAYS} days"
        ));
    }
    let member = member_of_organization(context, member).await?;

    let (rhythms, work_slots) = context
        .usecase
        .get_work_time(member, range)
        .await
        .map_err(failure)?;

    Ok(json!({
        "rhythms": rhythms.iter().map(rhythm_json).collect::<Vec<_>>(),
        "work_slots": work_slots.iter().map(work_slot_json).collect::<Vec<_>>(),
    }))
}

async fn put_rhythm(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = MemberId(context.uuid("member_id")?);
    let effective_from = date(context, "effective_from")?;
    let effective_to = optional_date(context, "effective_to")?;
    let slots: Vec<RhythmSlotRaw> = required_json(context, "slots")?;
    let member = member_of_organization(context, member).await?;

    context
        .usecase
        .replace_rhythm(
            context.actor(),
            context.organization_id,
            member,
            effective_from,
            effective_to,
            slots
                .into_iter()
                .map(|slot| RhythmSlotInput {
                    weekday: slot.weekday,
                    starts_minute: slot.starts_minute,
                    ends_minute: slot.ends_minute,
                })
                .collect(),
        )
        .await
        .map(|rhythm| rhythm_json(&rhythm))
        .map_err(failure)
}

async fn put_work_slots(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = MemberId(context.uuid("member_id")?);
    let from = date(context, "from")?;
    let to = date(context, "to")?;
    let slots: Vec<WorkSlotRaw> = required_json(context, "slots")?;
    let member = member_of_organization(context, member).await?;

    context
        .usecase
        .replace_work_slots(ReplaceWorkSlotsCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            member_id: member,
            from,
            to,
            slots: slots
                .into_iter()
                .map(|slot| WorkSlotInput {
                    work_date: slot.work_date,
                    starts_minute: slot.starts_minute,
                    ends_minute: slot.ends_minute,
                })
                .collect(),
        })
        .await
        .map(|slots| Value::Array(slots.iter().map(work_slot_json).collect()))
        .map_err(failure)
}

#[cfg(test)]
mod tests_planning_a;
