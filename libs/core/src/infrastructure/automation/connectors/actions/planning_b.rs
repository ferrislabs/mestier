use std::collections::HashMap;
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use common::CoreError;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::super::fields;
use super::{ActionContext, failure};
use crate::application::task::{BoardDrop, ParentScope, TaskFilter};
use crate::{
    AssigneeRef, CreateDoeCommand, CreateProjectCommand, CreateProjectTemplateCommand,
    CreateTaskCommand, CreateTaskCommentCommand, CreateTaskLabelCommand,
    CreateTaskRecurrenceCommand, CustomerContextId, CustomerId, DateRange, DeleteScope, Doe,
    DoeModelId, Equipment, EquipmentId, InstantiateProjectTemplateCommand, MemberId,
    PatchTaskCommand, PatchTaskRecurrenceCommand, PlanningEntry, PlanningView, Project, ProjectId,
    ProjectTemplate, ProjectTemplateId, ProjectTemplateTask, ProjectTemplateTaskShapeCommand,
    QuoteId, RecurrenceRule, ReplaceProjectTemplateTasksCommand, Task, TaskComment, TaskCommentId,
    TaskId, TaskLabel, TaskLabelId, TaskRecurrence, TaskRecurrenceId, TaskStatus, TimeRange,
    UpdateProjectCommand, UpdateProjectTemplateCommand, UpdateTaskCommentCommand,
    UpdateTaskLabelCommand, weekday_from_iso, weekday_to_iso,
};
use crate::{AvailabilityReport, ConflictKind, OrganizationId};

#[cfg(test)]
#[path = "tests_planning_b.rs"]
mod tests_planning_b;

const MAX_WINDOW_DAYS: i64 = 92;
const MAX_TEXT_CHARS: usize = 20_000;
const MAX_NAME_CHARS: usize = 200;
const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;
const STATUS_HINT: &str = "must be one of BACKLOG, PLANNED, IN_PROGRESS, DONE, CANCELLED";

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.planning.get_planning" => get_planning(context).await,
        "mestier.planning.get_availability" => get_availability(context).await,
        "mestier.project_template.list" => list_templates(context).await,
        "mestier.project_template.create" => create_template(context).await,
        "mestier.project_template.get" => get_template(context).await,
        "mestier.project_template.update" => update_template(context).await,
        "mestier.project_template.archive" => archive_template(context).await,
        "mestier.project_template.restore" => restore_template(context).await,
        "mestier.project_template.replace_tasks" => replace_template_tasks(context).await,
        "mestier.project_template.instantiate" => instantiate_template(context).await,
        "mestier.project.list" => list_projects(context).await,
        "mestier.project.create_project" => create_project(context).await,
        "mestier.project.get" => get_project(context).await,
        "mestier.project.update" => update_project(context).await,
        "mestier.project.archive" => archive_project(context).await,
        "mestier.project.restore" => restore_project(context).await,
        "mestier.project.get_channel" => get_channel(context).await,
        "mestier.project.create_channel" => create_channel(context).await,
        "mestier.project.get_doe" => get_doe(context).await,
        "mestier.project.create_doe" => create_doe(context).await,
        "mestier.task_label.list" => list_labels(context).await,
        "mestier.task_label.create" => create_label(context).await,
        "mestier.task_label.update" => update_label(context).await,
        "mestier.task_label.delete" => delete_label(context).await,
        "mestier.task_recurrence.list" => list_recurrences(context).await,
        "mestier.task_recurrence.create" => create_recurrence(context).await,
        "mestier.task_recurrence.update" => update_recurrence(context).await,
        "mestier.task_recurrence.delete" => delete_recurrence(context).await,
        "mestier.task.list" => list_tasks(context).await,
        "mestier.task.create" => create_task(context).await,
        "mestier.task.get" => get_task(context).await,
        "mestier.task.update" => update_task(context).await,
        "mestier.task.delete" => delete_task(context).await,
        "mestier.task.bulk_assign" => bulk_assign(context).await,
        "mestier.task_comment.list" => list_comments(context).await,
        "mestier.task_comment.create" => create_comment(context).await,
        "mestier.task_comment.update" => update_comment(context).await,
        "mestier.task_comment.delete" => delete_comment(context).await,
        _ => return None,
    })
}

fn foreign() -> String {
    failure(CoreError::NotFound)
}

fn within(context: &ActionContext<'_>, owner: OrganizationId) -> Result<(), String> {
    if owner == context.organization_id {
        Ok(())
    } else {
        Err(foreign())
    }
}

fn limit_of(context: &ActionContext<'_>) -> Result<u64, String> {
    Ok(context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT))
}

fn offset_of(context: &ActionContext<'_>) -> Result<u64, String> {
    Ok(context.optional_whole_number("offset")?.unwrap_or(0))
}

fn date_of(raw: &str, name: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| format!("`{name}` must be a date written YYYY-MM-DD"))
}

fn moment_of(raw: &str, name: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(raw.trim())
        .map(|moment| moment.with_timezone(&Utc))
        .map_err(|_| format!("`{name}` must be an ISO 8601 date and time"))
}

fn time_of(raw: &str, name: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(raw.trim(), "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(raw.trim(), "%H:%M"))
        .map_err(|_| format!("`{name}` must be a time written HH:MM"))
}

fn status_of(raw: &str) -> Result<TaskStatus, String> {
    TaskStatus::from_str(raw.trim()).map_err(|_| format!("`status` {STATUS_HINT}"))
}

fn uuid_of(raw: &str, name: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw.trim()).map_err(|_| format!("`{name}` must be a UUID"))
}

fn int_of(value: &Value, name: &str) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|number| i32::try_from(number).ok())
        .ok_or_else(|| format!("`{name}` must be a whole number"))
}

fn small_int_of(value: &Value, name: &str) -> Result<i16, String> {
    value
        .as_i64()
        .and_then(|number| i16::try_from(number).ok())
        .ok_or_else(|| format!("`{name}` must be a whole number"))
}

fn optional_int(context: &ActionContext<'_>, name: &str) -> Result<Option<i32>, String> {
    context
        .optional_whole_number(name)?
        .map(|number| i32::try_from(number).map_err(|_| format!("`{name}` must be a whole number")))
        .transpose()
}

fn patch<T>(
    context: &ActionContext<'_>,
    name: &str,
    parse: impl Fn(&str) -> Result<T, String>,
) -> Result<Option<Option<T>>, String> {
    match context.config.get(name) {
        None => Ok(None),
        Some(Value::Null) => Ok(Some(None)),
        Some(Value::String(raw)) if raw.trim().is_empty() => Ok(None),
        Some(Value::String(raw)) => parse(raw).map(|value| Some(Some(value))),
        Some(_) => Err(format!("`{name}` must resolve to a string")),
    }
}

fn patch_text(context: &ActionContext<'_>, name: &str) -> Result<Option<Option<String>>, String> {
    patch(context, name, |raw| Ok(raw.to_owned()))
}

fn patch_uuid(context: &ActionContext<'_>, name: &str) -> Result<Option<Option<Uuid>>, String> {
    patch(context, name, |raw| uuid_of(raw, name))
}

fn patch_moment(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<Option<Option<DateTime<Utc>>>, String> {
    patch(context, name, |raw| moment_of(raw, name))
}

fn patch_date(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<Option<Option<NaiveDate>>, String> {
    patch(context, name, |raw| date_of(raw, name))
}

fn uuid_list(context: &ActionContext<'_>, name: &str) -> Result<Option<Vec<Uuid>>, String> {
    match context.config.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .iter()
            .enumerate()
            .map(|(position, item)| {
                item.as_str()
                    .ok_or_else(|| format!("`{name}` element {position} must be a string"))
                    .and_then(|raw| uuid_of(raw, name))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(format!("`{name}` must resolve to an array")),
    }
}

fn required_uuid_list(context: &ActionContext<'_>, name: &str) -> Result<Vec<Uuid>, String> {
    uuid_list(context, name)?.ok_or_else(|| format!("`{name}` must resolve to an array"))
}

fn weekdays_of(context: &ActionContext<'_>) -> Result<Vec<chrono::Weekday>, String> {
    match context.config.get("weekdays") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_i64()
                    .and_then(|number| i16::try_from(number).ok())
                    .ok_or_else(|| "`weekdays` must hold whole numbers from 1 to 7".to_owned())
                    .and_then(|number| {
                        weekday_from_iso(number).map_err(|_| {
                            "`weekdays` must hold whole numbers from 1 to 7".to_owned()
                        })
                    })
            })
            .collect(),
        _ => Err("`weekdays` must resolve to an array".to_owned()),
    }
}

fn rule_of(context: &ActionContext<'_>, frequency: &str) -> Result<RecurrenceRule, String> {
    match frequency {
        "DAILY" => Ok(RecurrenceRule::Daily),
        "WEEKLY" => Ok(RecurrenceRule::Weekly {
            weekdays: weekdays_of(context)?,
        }),
        "MONTHLY" => {
            let day = context
                .optional_whole_number("day_of_month")?
                .and_then(|day| u8::try_from(day).ok())
                .ok_or_else(|| "`day_of_month` must be a whole number from 1 to 31".to_owned())?;
            Ok(RecurrenceRule::Monthly { day_of_month: day })
        }
        _ => Err("`frequency` must be one of DAILY, WEEKLY, MONTHLY".to_owned()),
    }
}

fn timezone_of(raw: &str) -> Result<chrono_tz::Tz, String> {
    raw.trim()
        .parse()
        .map_err(|_| "`timezone` must be an IANA time zone name".to_owned())
}

async fn check_targets(
    context: &ActionContext<'_>,
    customer_id: Option<CustomerId>,
    customer_context_id: Option<CustomerContextId>,
    quote_id: Option<QuoteId>,
    pair_required: bool,
) -> Result<(), String> {
    let check_customer = match (customer_id, customer_context_id) {
        (Some(_), _) if !pair_required => true,
        (Some(_), Some(_)) => true,
        _ => false,
    };
    if let (true, Some(customer_id)) = (check_customer, customer_id) {
        let customer = context
            .usecase
            .get_customer(customer_id)
            .await
            .map_err(failure)?;
        within(context, customer.organization_id)?;
        if let Some(customer_context_id) = customer_context_id {
            let found = context
                .usecase
                .get_customer_context(customer_context_id)
                .await
                .map_err(failure)?;
            if found.customer_id != customer_id {
                return Err(foreign());
            }
        }
    }
    if let Some(quote_id) = quote_id {
        let quote = context.usecase.get_quote(quote_id).await.map_err(failure)?;
        within(context, quote.organization_id)?;
    }
    Ok(())
}

type Targets = (
    Option<CustomerId>,
    Option<CustomerContextId>,
    Option<QuoteId>,
);

fn targets_of(context: &ActionContext<'_>) -> Result<Targets, String> {
    Ok((
        context.optional_uuid("customer_id")?.map(CustomerId),
        context
            .optional_uuid("customer_context_id")?
            .map(CustomerContextId),
        context.optional_uuid("quote_id")?.map(QuoteId),
    ))
}

fn label_json(label: &TaskLabel) -> Value {
    json!({
        "id": label.id,
        "organization_id": label.organization_id,
        "name": label.name,
        "color": label.color,
        "created_at": label.created_at,
        "updated_at": label.updated_at,
    })
}

fn equipment_json(equipment: &Equipment) -> Value {
    json!({
        "id": equipment.id,
        "organization_id": equipment.organization_id,
        "name": equipment.name,
        "hourly_rate_cents": equipment.hourly_rate_cents,
        "created_at": equipment.created_at,
        "updated_at": equipment.updated_at,
    })
}

fn task_json(
    task: &Task,
    child_count: Option<i64>,
    labels: &[TaskLabel],
    equipment: &[Equipment],
) -> Value {
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
        "member_ids": task.assignments.iter().map(|assignment| assignment.member_id).collect::<Vec<_>>(),
        "assignments": task
            .assignments
            .iter()
            .map(|assignment| json!({ "id": assignment.id, "member_id": assignment.member_id }))
            .collect::<Vec<_>>(),
        "child_count": child_count,
        "labels": labels.iter().map(label_json).collect::<Vec<_>>(),
        "equipment": equipment.iter().map(equipment_json).collect::<Vec<_>>(),
        "created_at": task.created_at,
        "updated_at": task.updated_at,
    })
}

async fn tasks_json(
    context: &ActionContext<'_>,
    tasks: &[Task],
    counts: Option<&HashMap<TaskId, i64>>,
) -> Result<Vec<Value>, String> {
    let ids: Vec<TaskId> = tasks.iter().map(|task| task.id).collect();
    let mut labels = context
        .usecase
        .list_task_labels_for_tasks(ids.clone())
        .await
        .map_err(failure)?;
    let mut equipment = context
        .usecase
        .list_equipment_for_tasks(ids)
        .await
        .map_err(failure)?;
    Ok(tasks
        .iter()
        .map(|task| {
            let child_count = counts.map(|counts| counts.get(&task.id).copied().unwrap_or(0));
            task_json(
                task,
                child_count,
                &labels.remove(&task.id).unwrap_or_default(),
                &equipment.remove(&task.id).unwrap_or_default(),
            )
        })
        .collect())
}

async fn one_task_json(
    context: &ActionContext<'_>,
    task: &Task,
    counts: Option<&HashMap<TaskId, i64>>,
) -> Result<Value, String> {
    let mut rows = tasks_json(context, std::slice::from_ref(task), counts).await?;
    rows.pop()
        .ok_or_else(|| "the task could not be rendered".to_owned())
}

fn project_json(project: &Project) -> Value {
    json!({
        "id": project.id,
        "organization_id": project.organization_id,
        "name": project.name,
        "customer_id": project.customer_id,
        "customer_context_id": project.customer_context_id,
        "quote_id": project.quote_id,
        "archived_at": project.archived_at,
        "is_internal": project.is_internal(),
        "created_at": project.created_at,
        "updated_at": project.updated_at,
    })
}

fn template_task_json(task: &ProjectTemplateTask) -> Value {
    json!({
        "id": task.id,
        "organization_id": task.organization_id,
        "template_id": task.template_id,
        "title": task.title,
        "description": task.description,
        "day_offset": task.day_offset,
        "starts_minute": task.starts_minute,
        "ends_minute": task.ends_minute,
        "all_day": task.all_day,
        "blocks_availability": task.blocks_availability,
        "expenses_cents": task.expenses_cents,
        "expenses_label": task.expenses_label,
        "parent_index": task.parent_index,
        "position": task.position,
    })
}

fn template_json(template: &ProjectTemplate, tasks: Option<&[ProjectTemplateTask]>) -> Value {
    let mut output = json!({
        "id": template.id,
        "organization_id": template.organization_id,
        "name": template.name,
        "description": template.description,
        "archived_at": template.archived_at,
        "created_at": template.created_at,
        "updated_at": template.updated_at,
    });
    if let Some(tasks) = tasks {
        output["tasks"] = Value::Array(tasks.iter().map(template_task_json).collect());
    }
    output
}

fn recurrence_json(recurrence: &TaskRecurrence) -> Value {
    let mut output = json!({
        "id": recurrence.id,
        "organization_id": recurrence.organization_id,
        "starts_on": recurrence.starts_on,
        "ends_on": recurrence.ends_on,
        "horizon_filled_to": recurrence.horizon_filled_to,
        "timezone": recurrence.timezone.to_string(),
        "start_time": recurrence.start_time,
        "duration_minutes": recurrence.duration_minutes,
        "all_day": recurrence.all_day,
        "title": recurrence.title,
        "description": recurrence.description,
        "blocks_availability": recurrence.blocks_availability,
        "customer_id": recurrence.customer_id,
        "customer_context_id": recurrence.customer_context_id,
        "project_id": recurrence.project_id,
        "assignee_member_ids": recurrence.assignee_member_ids,
        "created_at": recurrence.created_at,
        "updated_at": recurrence.updated_at,
    });
    match &recurrence.rule {
        RecurrenceRule::Daily => output["frequency"] = json!("DAILY"),
        RecurrenceRule::Weekly { weekdays } => {
            output["frequency"] = json!("WEEKLY");
            output["weekdays"] = json!(
                weekdays
                    .iter()
                    .map(|day| weekday_to_iso(*day))
                    .collect::<Vec<_>>()
            );
        }
        RecurrenceRule::Monthly { day_of_month } => {
            output["frequency"] = json!("MONTHLY");
            output["day_of_month"] = json!(day_of_month);
        }
    }
    output
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

fn planning_json(view: &PlanningView) -> Value {
    json!({
        "timezone": view.timezone,
        "resources": view
            .resources
            .iter()
            .map(|resource| json!({
                "resource_id": resource.resource_id(),
                "member_id": resource.member_id,
                "display_name": resource.display_name,
                "employee_id": resource.employee_id,
                "hourly_rate_cents": resource.hourly_rate_cents,
                "weekly_contract_minutes": resource.weekly_contract_minutes,
            }))
            .collect::<Vec<_>>(),
        "entries": view.entries.iter().map(entry_json).collect::<Vec<_>>(),
        "work_time": view
            .work_time
            .iter()
            .map(|member| json!({
                "member_id": member.member_id,
                "days": member
                    .days
                    .iter()
                    .map(|(date, intervals)| json!({
                        "date": date,
                        "intervals": intervals
                            .iter()
                            .map(|interval| json!({
                                "starts_minute": interval.starts_minute,
                                "ends_minute": interval.ends_minute,
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    })
}

fn entry_json(entry: &PlanningEntry) -> Value {
    match entry {
        PlanningEntry::Task {
            id,
            parent_task_id,
            recurrence_id,
            starts_at,
            ends_at,
            all_day,
            status,
            blocks_availability,
            child_count,
            title,
            customer_name,
            context_label,
            description,
            member_ids,
            labels,
        } => json!({
            "kind": "task",
            "id": id,
            "parent_task_id": parent_task_id,
            "recurrence_id": recurrence_id,
            "starts_at": starts_at,
            "ends_at": ends_at,
            "all_day": all_day,
            "status": status,
            "blocks_availability": blocks_availability,
            "child_count": child_count,
            "title": title,
            "customer_name": customer_name,
            "context_label": context_label,
            "description": description,
            "member_ids": member_ids,
            "labels": labels.iter().map(label_json).collect::<Vec<_>>(),
        }),
        PlanningEntry::Absence {
            id,
            starts_at,
            ends_at,
            all_day,
            absence_kind,
            note,
            member_id,
        } => json!({
            "kind": "absence",
            "id": id,
            "starts_at": starts_at,
            "ends_at": ends_at,
            "all_day": all_day,
            "absence_kind": absence_kind,
            "note": note,
            "member_id": member_id,
        }),
    }
}

fn availability_json(reports: &[AvailabilityReport]) -> Value {
    json!({
        "resources": reports
            .iter()
            .map(|report| json!({
                "resource_id": report.resource.resource_id(),
                "available": report.available(),
                "conflicts": report
                    .conflicts
                    .iter()
                    .map(|conflict| match &conflict.kind {
                        ConflictKind::Absence { kind, note } => json!({
                            "kind": "absence",
                            "reason": kind,
                            "note": note,
                            "starts_at": conflict.starts_at,
                            "ends_at": conflict.ends_at,
                        }),
                        ConflictKind::OutsideWorkHours => json!({
                            "kind": "outside_work_hours",
                            "starts_at": conflict.starts_at,
                            "ends_at": conflict.ends_at,
                        }),
                        ConflictKind::OverlappingTask { task_id } => json!({
                            "kind": "overlapping_task",
                            "task_id": task_id,
                            "starts_at": conflict.starts_at,
                            "ends_at": conflict.ends_at,
                        }),
                    })
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    })
}

async fn get_planning(context: &ActionContext<'_>) -> Result<Value, String> {
    let from = date_of(context.text("from")?, "from")?;
    let to = date_of(context.text("to")?, "to")?;
    let range = DateRange::new(from, to).map_err(failure)?;
    if range.span_days() > MAX_WINDOW_DAYS {
        return Err(format!(
            "planning window must not exceed {MAX_WINDOW_DAYS} days"
        ));
    }

    context
        .usecase
        .get_planning(context.organization_id, range)
        .await
        .map(|view| planning_json(&view))
        .map_err(failure)
}

async fn get_availability(context: &ActionContext<'_>) -> Result<Value, String> {
    let starts_at = moment_of(context.text("starts_at")?, "starts_at")?;
    let ends_at = moment_of(context.text("ends_at")?, "ends_at")?;
    let window = TimeRange::new(starts_at, ends_at).map_err(failure)?;
    if window.ends_at - window.starts_at > chrono::Duration::days(MAX_WINDOW_DAYS) {
        return Err(format!(
            "planning window must not exceed {MAX_WINDOW_DAYS} days"
        ));
    }

    context
        .usecase
        .get_availability(context.organization_id, window)
        .await
        .map(|reports| availability_json(&reports))
        .map_err(failure)
}

fn template_id(context: &ActionContext<'_>) -> Result<ProjectTemplateId, String> {
    context.uuid("template_id").map(ProjectTemplateId)
}

async fn owned_template(
    context: &ActionContext<'_>,
    id: ProjectTemplateId,
) -> Result<ProjectTemplate, String> {
    let template = context
        .usecase
        .get_project_template(id)
        .await
        .map_err(failure)?;
    within(context, template.organization_id)?;
    Ok(template)
}

fn shape_of(position: usize, item: &Value) -> Result<ProjectTemplateTaskShapeCommand, String> {
    let object = item
        .as_object()
        .ok_or_else(|| format!("`tasks` element {position} must be an object"))?;
    let title = fields::filled(object, "title", MAX_NAME_CHARS)
        .map_err(|error| format!("`tasks` element {position}: {error}"))?
        .to_owned();
    let number = |name: &str| -> Result<Option<&Value>, String> {
        Ok(object.get(name).filter(|value| !value.is_null()))
    };
    let day_offset = number("day_offset")?
        .ok_or_else(|| format!("`tasks` element {position}: `day_offset` is required"))
        .and_then(|value| int_of(value, "day_offset"))?;
    let flag = |name: &str, default: Option<bool>| -> Result<bool, String> {
        match object.get(name) {
            None | Some(Value::Null) => {
                default.ok_or_else(|| format!("`tasks` element {position}: `{name}` is required"))
            }
            Some(Value::Bool(value)) => Ok(*value),
            Some(_) => Err(format!(
                "`tasks` element {position}: `{name}` must be true or false"
            )),
        }
    };
    let text = |name: &str| -> Result<Option<String>, String> {
        match object.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(value)) => Ok(Some(value.clone())),
            Some(_) => Err(format!(
                "`tasks` element {position}: `{name}` must resolve to a string"
            )),
        }
    };
    Ok(ProjectTemplateTaskShapeCommand {
        title,
        description: text("description")?,
        day_offset,
        starts_minute: number("starts_minute")?
            .map(|value| small_int_of(value, "starts_minute"))
            .transpose()?,
        ends_minute: number("ends_minute")?
            .map(|value| small_int_of(value, "ends_minute"))
            .transpose()?,
        all_day: flag("all_day", Some(false))?,
        blocks_availability: flag("blocks_availability", None)?,
        expenses_cents: number("expenses_cents")?
            .map(|value| int_of(value, "expenses_cents"))
            .transpose()?
            .unwrap_or(0),
        expenses_label: text("expenses_label")?,
        parent_index: number("parent_index")?
            .map(|value| int_of(value, "parent_index"))
            .transpose()?,
    })
}

fn shapes_of(
    config: &Map<String, Value>,
    required: bool,
) -> Result<Vec<ProjectTemplateTaskShapeCommand>, String> {
    match config.get("tasks") {
        None | Some(Value::Null) if !required => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .enumerate()
            .map(|(position, item)| shape_of(position, item))
            .collect(),
        _ => Err("`tasks` must resolve to an array".to_owned()),
    }
}

async fn list_templates(context: &ActionContext<'_>) -> Result<Value, String> {
    let include_archived = context.optional_bool("include_archived")?.unwrap_or(false);
    let limit = limit_of(context)?;
    let offset = offset_of(context)?;

    let (templates, _) = context
        .usecase
        .list_project_templates(context.organization_id, include_archived, limit, offset)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        templates
            .iter()
            .map(|template| template_json(template, None))
            .collect(),
    ))
}

async fn create_template(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let description = context.optional_text("description")?.map(str::to_owned);
    let tasks = shapes_of(context.config, false)?;

    let template = context
        .usecase
        .create_project_template(CreateProjectTemplateCommand {
            organization_id: context.organization_id,
            name,
            description,
            tasks,
        })
        .await
        .map_err(failure)?;
    let tasks = context
        .usecase
        .list_project_template_tasks(template.id)
        .await
        .map_err(failure)?;

    Ok(template_json(&template, Some(&tasks)))
}

async fn get_template(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = template_id(context)?;
    let template = owned_template(context, id).await?;
    let tasks = context
        .usecase
        .list_project_template_tasks(id)
        .await
        .map_err(failure)?;

    Ok(template_json(&template, Some(&tasks)))
}

async fn update_template(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = template_id(context)?;
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let description = context.optional_text("description")?.map(str::to_owned);
    owned_template(context, id).await?;

    let template = context
        .usecase
        .update_project_template(UpdateProjectTemplateCommand {
            id,
            name,
            description,
        })
        .await
        .map_err(failure)?;

    Ok(template_json(&template, None))
}

async fn archive_template(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = template_id(context)?;
    owned_template(context, id).await?;

    context
        .usecase
        .archive_project_template(id)
        .await
        .map_err(failure)?;

    Ok(json!({ "template_id": id, "archived": true }))
}

async fn restore_template(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = template_id(context)?;
    owned_template(context, id).await?;

    context
        .usecase
        .restore_project_template(id)
        .await
        .map_err(failure)?;
    let template = context
        .usecase
        .get_project_template(id)
        .await
        .map_err(failure)?;

    Ok(template_json(&template, None))
}

async fn replace_template_tasks(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = template_id(context)?;
    let tasks = shapes_of(context.config, true)?;
    owned_template(context, id).await?;

    context
        .usecase
        .replace_project_template_tasks(ReplaceProjectTemplateTasksCommand {
            template_id: id,
            tasks,
        })
        .await
        .map(|tasks| Value::Array(tasks.iter().map(template_task_json).collect()))
        .map_err(failure)
}

async fn instantiate_template(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = template_id(context)?;
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let start_date = date_of(context.text("start_date")?, "start_date")?;
    let (customer_id, customer_context_id, quote_id) = targets_of(context)?;
    owned_template(context, id).await?;
    check_targets(context, customer_id, customer_context_id, quote_id, false).await?;

    let (project, tasks) = context
        .usecase
        .instantiate_project_template(InstantiateProjectTemplateCommand {
            template_id: id,
            organization_id: context.organization_id,
            name,
            start_date,
            customer_id,
            customer_context_id,
            quote_id,
        })
        .await
        .map_err(failure)?;

    Ok(json!({
        "project": project_json(&project),
        "tasks": tasks_json(context, &tasks, None).await?,
    }))
}

fn project_id(context: &ActionContext<'_>) -> Result<ProjectId, String> {
    context.uuid("project_id").map(ProjectId)
}

async fn owned_project(context: &ActionContext<'_>, id: ProjectId) -> Result<Project, String> {
    let project = context.usecase.get_project(id).await.map_err(failure)?;
    within(context, project.organization_id)?;
    Ok(project)
}

async fn list_projects(context: &ActionContext<'_>) -> Result<Value, String> {
    let customer_id = context.optional_uuid("customer_id")?.map(CustomerId);
    let quote_id = context.optional_uuid("quote_id")?.map(QuoteId);
    let include_archived = context.optional_bool("include_archived")?.unwrap_or(false);
    let limit = limit_of(context)?;
    let offset = offset_of(context)?;

    let (projects, _) = context
        .usecase
        .list_projects(
            context.organization_id,
            customer_id,
            quote_id,
            include_archived,
            limit,
            offset,
        )
        .await
        .map_err(failure)?;

    Ok(Value::Array(projects.iter().map(project_json).collect()))
}

async fn create_project(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let (customer_id, customer_context_id, quote_id) = targets_of(context)?;
    check_targets(context, customer_id, customer_context_id, quote_id, false).await?;

    context
        .usecase
        .create_project(CreateProjectCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            name,
            customer_id,
            customer_context_id,
            quote_id,
        })
        .await
        .map(|project| project_json(&project))
        .map_err(failure)
}

async fn get_project(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;

    owned_project(context, id)
        .await
        .map(|project| project_json(&project))
}

async fn update_project(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let (customer_id, customer_context_id, quote_id) = targets_of(context)?;
    owned_project(context, id).await?;
    check_targets(context, customer_id, customer_context_id, quote_id, false).await?;

    context
        .usecase
        .update_project(UpdateProjectCommand {
            actor: context.actor(),
            id,
            name,
            customer_id,
            customer_context_id,
            quote_id,
        })
        .await
        .map(|project| project_json(&project))
        .map_err(failure)
}

async fn archive_project(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    owned_project(context, id).await?;

    context
        .usecase
        .archive_project(context.actor(), id)
        .await
        .map_err(failure)?;

    Ok(json!({ "project_id": id, "archived": true }))
}

async fn restore_project(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    owned_project(context, id).await?;

    context
        .usecase
        .restore_project(context.actor(), id)
        .await
        .map_err(failure)?;

    owned_project(context, id)
        .await
        .map(|project| project_json(&project))
}

fn channel_json(channel: &discord::Channel) -> Value {
    json!({
        "id": channel.id,
        "organization_id": channel.organization_id,
        "project_id": channel.project_id,
        "name": channel.name,
        "topic": channel.topic,
        "archived": channel.archived,
        "created_at": channel.created_at,
        "updated_at": channel.updated_at,
    })
}

async fn get_channel(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    owned_project(context, id).await?;

    context
        .usecase
        .get_project_channel(id)
        .await
        .map(|channel| channel_json(&channel))
        .map_err(failure)
}

async fn create_channel(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    let name = context.optional_text("name")?.map(str::to_owned);
    owned_project(context, id).await?;

    context
        .usecase
        .create_project_channel(id, name)
        .await
        .map(|channel| channel_json(&channel))
        .map_err(failure)
}

async fn get_doe(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    owned_project(context, id).await?;

    let doe = context.usecase.get_project_doe(id).await.map_err(failure)?;
    within(context, doe.organization_id)?;

    Ok(doe_json(&doe))
}

async fn create_doe(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = project_id(context)?;
    let model_id = context.uuid("model_id").map(DoeModelId)?;
    owned_project(context, id).await?;

    context
        .usecase
        .create_doe(CreateDoeCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            project_id: id,
            model_id,
        })
        .await
        .map(|doe| doe_json(&doe))
        .map_err(failure)
}

fn label_id(context: &ActionContext<'_>) -> Result<TaskLabelId, String> {
    context.uuid("label_id").map(TaskLabelId)
}

async fn owned_label(context: &ActionContext<'_>, id: TaskLabelId) -> Result<TaskLabel, String> {
    let label = context.usecase.get_task_label(id).await.map_err(failure)?;
    within(context, label.organization_id)?;
    Ok(label)
}

async fn list_labels(context: &ActionContext<'_>) -> Result<Value, String> {
    let limit = limit_of(context)?;

    let labels = context
        .usecase
        .list_task_labels(context.organization_id)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        labels
            .iter()
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .map(label_json)
            .collect(),
    ))
}

async fn create_label(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let color = context.filled("color", MAX_NAME_CHARS)?.to_owned();

    context
        .usecase
        .create_task_label(CreateTaskLabelCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            name,
            color,
        })
        .await
        .map(|label| label_json(&label))
        .map_err(failure)
}

async fn update_label(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = label_id(context)?;
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let color = context.filled("color", MAX_NAME_CHARS)?.to_owned();
    owned_label(context, id).await?;

    context
        .usecase
        .update_task_label(UpdateTaskLabelCommand {
            actor: context.actor(),
            id,
            name,
            color,
        })
        .await
        .map(|label| label_json(&label))
        .map_err(failure)
}

async fn delete_label(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = label_id(context)?;
    owned_label(context, id).await?;

    context
        .usecase
        .delete_task_label(context.actor(), id)
        .await
        .map_err(failure)?;

    Ok(json!({ "label_id": id, "deleted": true }))
}

fn recurrence_id(context: &ActionContext<'_>) -> Result<TaskRecurrenceId, String> {
    context.uuid("recurrence_id").map(TaskRecurrenceId)
}

async fn owned_recurrence(
    context: &ActionContext<'_>,
    id: TaskRecurrenceId,
) -> Result<TaskRecurrence, String> {
    let recurrence = context
        .usecase
        .get_task_recurrence(id)
        .await
        .map_err(failure)?;
    within(context, recurrence.organization_id)?;
    Ok(recurrence)
}

fn members_of(ids: Vec<Uuid>) -> Vec<MemberId> {
    ids.into_iter().map(MemberId).collect()
}

async fn list_recurrences(context: &ActionContext<'_>) -> Result<Value, String> {
    let limit = limit_of(context)?;

    let recurrences = context
        .usecase
        .list_task_recurrences(context.organization_id)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        recurrences
            .iter()
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .map(recurrence_json)
            .collect(),
    ))
}

async fn create_recurrence(context: &ActionContext<'_>) -> Result<Value, String> {
    let rule = rule_of(context, context.text("frequency")?)?;
    let starts_on = date_of(context.text("starts_on")?, "starts_on")?;
    let ends_on = context
        .optional_text("ends_on")?
        .map(|raw| date_of(raw, "ends_on"))
        .transpose()?;
    let timezone = timezone_of(context.text("timezone")?)?;
    let start_time = time_of(context.text("start_time")?, "start_time")?;
    let duration_minutes = optional_int(context, "duration_minutes")?
        .ok_or_else(|| "`duration_minutes` must be a whole number".to_owned())?;
    let all_day = context.optional_bool("all_day")?.unwrap_or(false);
    let title = context.filled("title", MAX_NAME_CHARS)?.to_owned();
    let description = context.optional_text("description")?.map(str::to_owned);
    let blocks_availability = context
        .optional_bool("blocks_availability")?
        .ok_or_else(|| "`blocks_availability` must be true or false".to_owned())?;
    let customer_id = context.optional_uuid("customer_id")?.map(CustomerId);
    let customer_context_id = context
        .optional_uuid("customer_context_id")?
        .map(CustomerContextId);
    let project_id = context.optional_uuid("project_id")?.map(ProjectId);
    let assignees = members_of(uuid_list(context, "assignee_member_ids")?.unwrap_or_default());
    check_targets(context, customer_id, customer_context_id, None, true).await?;
    if let Some(project_id) = project_id {
        owned_project(context, project_id).await?;
    }

    context
        .usecase
        .create_task_recurrence(CreateTaskRecurrenceCommand {
            organization_id: context.organization_id,
            rule,
            starts_on,
            ends_on,
            timezone,
            start_time,
            duration_minutes,
            all_day,
            title,
            description,
            blocks_availability,
            customer_id,
            customer_context_id,
            project_id,
            assignee_member_ids: assignees,
        })
        .await
        .map(|recurrence| recurrence_json(&recurrence))
        .map_err(failure)
}

async fn update_recurrence(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = recurrence_id(context)?;
    let mut command = PatchTaskRecurrenceCommand::new(id);
    command.rule = context
        .optional_text("frequency")?
        .map(|frequency| rule_of(context, frequency))
        .transpose()?;
    command.ends_on = patch_date(context, "ends_on")?;
    command.start_time = context
        .optional_text("start_time")?
        .map(|raw| time_of(raw, "start_time"))
        .transpose()?;
    command.duration_minutes = optional_int(context, "duration_minutes")?;
    command.all_day = context.optional_bool("all_day")?;
    command.title = context
        .optional_text("title")?
        .map(|_| context.filled("title", MAX_NAME_CHARS).map(str::to_owned))
        .transpose()?;
    command.description = patch_text(context, "description")?;
    command.blocks_availability = context.optional_bool("blocks_availability")?;
    command.project_id = patch_uuid(context, "project_id")?.map(|id| id.map(ProjectId));
    command.assignee_member_ids = uuid_list(context, "assignee_member_ids")?.map(members_of);
    owned_recurrence(context, id).await?;
    if let Some(Some(project_id)) = command.project_id {
        owned_project(context, project_id).await?;
    }

    context
        .usecase
        .patch_task_recurrence(command)
        .await
        .map(|recurrence| recurrence_json(&recurrence))
        .map_err(failure)
}

async fn delete_recurrence(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = recurrence_id(context)?;
    owned_recurrence(context, id).await?;

    context
        .usecase
        .delete_task_recurrence(id)
        .await
        .map_err(failure)?;

    Ok(json!({ "recurrence_id": id, "deleted": true }))
}

fn task_id(context: &ActionContext<'_>) -> Result<TaskId, String> {
    context.uuid("task_id").map(TaskId)
}

async fn owned_task(context: &ActionContext<'_>, id: TaskId) -> Result<Task, String> {
    let task = context.usecase.get_task(id).await.map_err(failure)?;
    within(context, task.organization_id)?;
    Ok(task)
}

fn statuses_of(context: &ActionContext<'_>) -> Result<Option<Vec<TaskStatus>>, String> {
    let statuses = context
        .optional_strings("statuses")?
        .into_iter()
        .map(status_of)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((!statuses.is_empty()).then_some(statuses))
}

async fn list_tasks(context: &ActionContext<'_>) -> Result<Value, String> {
    let parent_task_id = context.optional_uuid("parent_task_id")?.map(TaskId);
    let project_id = context.optional_uuid("project_id")?.map(ProjectId);
    let statuses = statuses_of(context)?;
    let assignee_id = context.optional_uuid("assignee_member_id")?.map(MemberId);
    let label_id = context.optional_uuid("label_id")?.map(TaskLabelId);
    let customer_id = context.optional_uuid("customer_id")?.map(CustomerId);
    let title_contains = context.optional_text("q")?.map(str::to_owned);
    let unscheduled = context.optional_bool("unscheduled")?;
    let limit = limit_of(context)?;
    let offset = offset_of(context)?;

    let narrows = project_id.is_some()
        || statuses.is_some()
        || assignee_id.is_some()
        || label_id.is_some()
        || customer_id.is_some()
        || title_contains.is_some()
        || unscheduled.is_some();
    let parent = match parent_task_id {
        Some(id) => ParentScope::ChildrenOf(id),
        None if narrows => ParentScope::Any,
        None => ParentScope::Roots,
    };

    let (tasks, counts, _) = context
        .usecase
        .list_tasks(
            context.organization_id,
            TaskFilter {
                parent,
                project_id,
                statuses,
                assignee_id,
                label_id,
                customer_id,
                title_contains,
                unscheduled,
            },
            limit,
            offset,
        )
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        tasks_json(context, &tasks, Some(&counts)).await?,
    ))
}

async fn create_task(context: &ActionContext<'_>) -> Result<Value, String> {
    let title = context.filled("title", MAX_NAME_CHARS)?.to_owned();
    let description = context.optional_text("description")?.map(str::to_owned);
    let parent_task_id = context.optional_uuid("parent_task_id")?.map(TaskId);
    let starts_at = context
        .optional_text("starts_at")?
        .map(|raw| moment_of(raw, "starts_at"))
        .transpose()?;
    let ends_at = context
        .optional_text("ends_at")?
        .map(|raw| moment_of(raw, "ends_at"))
        .transpose()?;
    let all_day = context.optional_bool("all_day")?.unwrap_or(false);
    let status = context
        .optional_text("status")?
        .map(status_of)
        .transpose()?;
    let blocks_availability = context
        .optional_bool("blocks_availability")?
        .ok_or_else(|| "`blocks_availability` must be true or false".to_owned())?;
    let (customer_id, customer_context_id, quote_id) = targets_of(context)?;
    let project_id = context.optional_uuid("project_id")?.map(ProjectId);
    let expenses_cents = optional_int(context, "expenses_cents")?.unwrap_or(0);
    let expenses_label = context.optional_text("expenses_label")?.map(str::to_owned);
    if let Some(text) = &description
        && text.chars().count() > MAX_TEXT_CHARS
    {
        return Err(format!(
            "`description` holds at most {MAX_TEXT_CHARS} characters"
        ));
    }
    check_targets(context, customer_id, customer_context_id, quote_id, true).await?;

    let task = context
        .usecase
        .create_task(CreateTaskCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            parent_task_id,
            title,
            description,
            starts_at,
            ends_at,
            all_day,
            status,
            blocks_availability,
            customer_id,
            customer_context_id,
            quote_id,
            project_id,
            expenses_cents,
            expenses_label,
        })
        .await
        .map_err(failure)?;

    one_task_json(context, &task, None).await
}

async fn get_task(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = task_id(context)?;
    let task = owned_task(context, id).await?;

    one_task_json(context, &task, None).await
}

async fn update_task(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = task_id(context)?;
    let mut command = PatchTaskCommand::new(id, context.actor());
    command.title = context
        .optional_text("title")?
        .map(|_| context.filled("title", MAX_NAME_CHARS).map(str::to_owned))
        .transpose()?;
    command.description = patch_text(context, "description")?;
    command.parent_task_id = patch_uuid(context, "parent_task_id")?.map(|id| id.map(TaskId));
    command.starts_at = patch_moment(context, "starts_at")?;
    command.ends_at = patch_moment(context, "ends_at")?;
    command.all_day = context.optional_bool("all_day")?;
    command.status = context
        .optional_text("status")?
        .map(status_of)
        .transpose()?;
    command.blocks_availability = context.optional_bool("blocks_availability")?;
    command.assignees = uuid_list(context, "assignee_member_ids")?.map(|ids| {
        ids.into_iter()
            .map(|id| AssigneeRef(MemberId(id)))
            .collect()
    });
    command.label_ids =
        uuid_list(context, "label_ids")?.map(|ids| ids.into_iter().map(TaskLabelId).collect());
    command.equipment_ids =
        uuid_list(context, "equipment_ids")?.map(|ids| ids.into_iter().map(EquipmentId).collect());
    command.project_id = patch_uuid(context, "project_id")?.map(|id| id.map(ProjectId));
    command.expenses_cents = optional_int(context, "expenses_cents")?;
    command.expenses_label = patch_text(context, "expenses_label")?;
    let preceding = patch_uuid(context, "preceding_task_id")?;
    let following = patch_uuid(context, "following_task_id")?;
    if preceding.is_some() || following.is_some() {
        command.board_drop = Some(BoardDrop {
            preceding: preceding.flatten().map(TaskId),
            following: following.flatten().map(TaskId),
        });
    }
    let before = owned_task(context, id).await?;
    if let Some(Some(project_id)) = command.project_id {
        owned_project(context, project_id).await?;
    }

    let task = context.usecase.patch_task(command).await.map_err(failure)?;

    Ok(json!({
        "task": one_task_json(context, &task, None).await?,
        "detached": before.recurrence_id.is_some(),
    }))
}

async fn delete_task(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = task_id(context)?;
    let scope = match context.optional_text("scope")? {
        None | Some("this_occurrence") => DeleteScope::ThisOccurrence,
        Some("this_and_following") => DeleteScope::ThisAndFollowing,
        Some(_) => {
            return Err("`scope` must be one of this_occurrence, this_and_following".to_owned());
        }
    };
    owned_task(context, id).await?;

    context
        .usecase
        .soft_delete_task_occurrence(context.actor(), id, scope)
        .await
        .map_err(failure)?;

    let scope_name = match scope {
        DeleteScope::ThisOccurrence => "this_occurrence",
        DeleteScope::ThisAndFollowing => "this_and_following",
    };
    Ok(json!({ "task_id": id, "scope": scope_name, "deleted": true }))
}

async fn bulk_assign(context: &ActionContext<'_>) -> Result<Value, String> {
    let task_ids: Vec<TaskId> = required_uuid_list(context, "task_ids")?
        .into_iter()
        .map(TaskId)
        .collect();
    let assignees: Vec<AssigneeRef> = required_uuid_list(context, "assignee_member_ids")?
        .into_iter()
        .map(|id| AssigneeRef(MemberId(id)))
        .collect();

    let tasks = context
        .usecase
        .bulk_assign_tasks(
            context.actor(),
            context.organization_id,
            task_ids,
            assignees,
        )
        .await
        .map_err(failure)?;

    Ok(json!({ "tasks": tasks_json(context, &tasks, None).await? }))
}

async fn list_comments(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = task_id(context)?;
    let limit = limit_of(context)?;
    let offset = offset_of(context)?;
    owned_task(context, id).await?;

    let (comments, _) = context
        .usecase
        .list_task_comments(id, limit, offset)
        .await
        .map_err(failure)?;
    let authors = context
        .usecase
        .list_task_comment_authors(
            comments
                .iter()
                .map(|comment| comment.author_user_id)
                .collect(),
        )
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        comments
            .iter()
            .map(|comment| {
                let display_name = authors
                    .get(&comment.author_user_id)
                    .map(|user| user.name.as_str())
                    .unwrap_or("Unknown");
                json!({
                    "id": comment.id,
                    "organization_id": comment.organization_id,
                    "task_id": comment.task_id,
                    "author": { "id": comment.author_user_id, "display_name": display_name },
                    "body": comment.body,
                    "created_at": comment.created_at,
                    "updated_at": comment.updated_at,
                    "author_is_self": false,
                })
            })
            .collect(),
    ))
}

const AUTOMATION_AUTHOR_NAME: &str = "Automatisation";

fn comment_json(comment: &TaskComment) -> Value {
    json!({
        "id": comment.id,
        "organization_id": comment.organization_id,
        "task_id": comment.task_id,
        "author": { "id": comment.author_user_id, "display_name": AUTOMATION_AUTHOR_NAME },
        "body": comment.body,
        "created_at": comment.created_at,
        "updated_at": comment.updated_at,
        "author_is_self": true,
    })
}

fn comment_id(context: &ActionContext<'_>) -> Result<TaskCommentId, String> {
    context.uuid("comment_id").map(TaskCommentId)
}

async fn owned_comment(
    context: &ActionContext<'_>,
    id: TaskCommentId,
) -> Result<TaskComment, String> {
    let comment = context
        .usecase
        .get_task_comment(id)
        .await
        .map_err(failure)?;
    within(context, comment.organization_id)?;
    Ok(comment)
}

async fn create_comment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = task_id(context)?;
    let body = context.filled("body", MAX_TEXT_CHARS)?;
    owned_task(context, id).await?;
    let author = context.system_user_id().await?;

    context
        .usecase
        .create_task_comment(CreateTaskCommentCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            task_id: id,
            author_user_id: author,
            body: body.to_owned(),
        })
        .await
        .map(|comment| comment_json(&comment))
        .map_err(failure)
}

async fn update_comment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = comment_id(context)?;
    let body = context.filled("body", MAX_TEXT_CHARS)?;
    owned_comment(context, id).await?;
    let author = context.system_user_id().await?;

    context
        .usecase
        .update_task_comment(UpdateTaskCommentCommand {
            actor: context.actor(),
            id,
            acting_user_id: author,
            body: body.to_owned(),
        })
        .await
        .map(|comment| comment_json(&comment))
        .map_err(failure)
}

async fn delete_comment(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = comment_id(context)?;
    owned_comment(context, id).await?;
    let author = context.system_user_id().await?;

    context
        .usecase
        .delete_task_comment(context.actor(), id, author)
        .await
        .map_err(failure)?;

    Ok(json!({ "comment_id": id, "deleted": true }))
}
