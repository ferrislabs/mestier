use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, UserId};
use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::application::policy::user_subject;
use crate::domain::assignment_report::commands::{
    AmendAssignmentReportCommand, ReportAssignmentCommand, WithdrawAssignmentReportCommand,
};
use crate::domain::maintenance::commands::ProposeInterventionCommand;
use crate::domain::time_entry::commands::{
    AttachTimeEntryPhotoCommand, DeclareTimeEntryCommand, StartTimeEntryCommand,
};
use crate::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, DayLog, EmployeeId,
    Intervention, MemberId, Task, TaskAssignmentId, TaskId, TimeEntry, TimeEntryId, TimeEntryPhoto,
    TimeEntryPhotoPhase,
};

#[cfg(test)]
mod tests_terrain;

const MAX_COMMENT_CHARS: usize = 2_000;
const MAX_STORAGE_KEY_CHARS: usize = 500;
const MAX_MINUTES: u64 = 24 * 60;
const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.terrain.list_tasks" => list_tasks(context).await,
        "mestier.terrain.get_current" => get_current(context).await,
        "mestier.terrain.start_time_entry" => start_time_entry(context).await,
        "mestier.terrain.declare_time_entry" => declare_time_entry(context).await,
        "mestier.terrain.stop_time_entry" => stop_time_entry(context).await,
        "mestier.terrain.recover_time_entry" => recover_time_entry(context).await,
        "mestier.terrain.attach_time_entry_photo" => attach_photo(context).await,
        "mestier.terrain.end_day" => end_day(context).await,
        "mestier.terrain.report_assignment" => report_assignment(context).await,
        "mestier.terrain.amend_assignment_report" => amend_report(context).await,
        "mestier.terrain.withdraw_assignment_report" => withdraw_report(context).await,
        "mestier.terrain.list_assignment_reports" => list_reports(context).await,
        "mestier.terrain.propose_maintenance_intervention" => propose_intervention(context).await,
        _ => return None,
    })
}

fn not_found() -> String {
    failure(CoreError::NotFound)
}

fn moment(context: &ActionContext<'_>, name: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(context.text(name)?.trim())
        .map(|moment| moment.with_timezone(&Utc))
        .map_err(|_| format!("`{name}` must be an RFC 3339 date and time"))
}

fn optional_moment(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    context
        .optional_text(name)?
        .map(|raw| {
            DateTime::parse_from_rfc3339(raw)
                .map(|moment| moment.with_timezone(&Utc))
                .map_err(|_| format!("`{name}` must be an RFC 3339 date and time"))
        })
        .transpose()
}

fn date(raw: &str, name: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map_err(|_| format!("`{name}` must be a date written YYYY-MM-DD"))
}

fn minutes(context: &ActionContext<'_>) -> Result<u32, String> {
    let raw = context
        .optional_whole_number("reported_minutes")?
        .ok_or_else(|| "`reported_minutes` must resolve to a whole number".to_owned())?;
    if raw > MAX_MINUTES {
        return Err(format!("`reported_minutes` holds at most {MAX_MINUTES}"));
    }
    u32::try_from(raw).map_err(|_| "`reported_minutes` is too large".to_owned())
}

fn comment(context: &ActionContext<'_>) -> Result<Option<String>, String> {
    let comment = context.optional_text("comment")?;
    if comment.is_some_and(|text| text.chars().count() > MAX_COMMENT_CHARS) {
        return Err(format!(
            "`comment` holds at most {MAX_COMMENT_CHARS} characters"
        ));
    }
    Ok(comment.map(str::to_owned))
}

fn entry_json(entry: &TimeEntry) -> Value {
    json!({
        "id": entry.id,
        "organization_id": entry.organization_id,
        "task_id": entry.task_id,
        "employee_id": entry.employee_id,
        "started_at": entry.started_at,
        "ended_at": entry.ended_at,
        "worked_minutes": entry.worked_minutes(),
        "photos": entry.photos.iter().map(photo_json).collect::<Vec<_>>(),
    })
}

fn photo_json(photo: &TimeEntryPhoto) -> Value {
    json!({
        "id": photo.id,
        "time_entry_id": photo.time_entry_id,
        "phase": photo.phase,
        "storage_key": photo.storage_key,
        "created_at": photo.created_at,
    })
}

fn day_log_json(log: &DayLog) -> Value {
    json!({
        "id": log.id,
        "organization_id": log.organization_id,
        "employee_id": log.employee_id,
        "work_date": log.work_date,
        "ended_at": log.ended_at,
    })
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

fn intervention_json(intervention: &Intervention) -> Value {
    json!({
        "id": intervention.id,
        "line_id": intervention.line_id,
        "task_id": intervention.task_id,
        "done_on": intervention.done_on,
        "status": intervention.status,
    })
}

fn task_json(task: &Task, member: MemberId) -> Value {
    let assignment = task
        .assignments
        .iter()
        .find(|assignment| assignment.member_id == member)
        .map(|assignment| assignment.id);
    json!({
        "id": task.id,
        "title": task.title,
        "description": task.description,
        "starts_at": task.starts_at,
        "ends_at": task.ends_at,
        "all_day": task.all_day,
        "status": task.status,
        "customer_id": task.customer_id,
        "customer_context_id": task.customer_context_id,
        "maintenance_line_id": task.maintenance_line_id,
        "task_assignment_id": assignment,
    })
}

fn member_id(context: &ActionContext<'_>) -> Result<MemberId, String> {
    context.uuid("member_id").map(MemberId)
}

async fn seat_of(context: &ActionContext<'_>, member: MemberId) -> Result<crate::Member, String> {
    let seat = context
        .usecase
        .find_member_seat(member)
        .await
        .map_err(failure)?
        .ok_or_else(not_found)?;
    if seat.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(seat)
}

async fn employee_of(context: &ActionContext<'_>, member: MemberId) -> Result<EmployeeId, String> {
    seat_of(context, member).await?;
    let employee = context
        .usecase
        .get_employee_by_member(member)
        .await
        .map_err(failure)?;
    if employee.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(employee.id)
}

async fn entry_of(context: &ActionContext<'_>, id: TimeEntryId) -> Result<TimeEntry, String> {
    let entry = context
        .usecase
        .get_time_entry(id)
        .await
        .map_err(failure)?
        .ok_or_else(not_found)?;
    if entry.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(entry)
}

async fn report_of(
    context: &ActionContext<'_>,
    id: AssignmentReportId,
) -> Result<AssignmentReport, String> {
    let report = context
        .usecase
        .get_assignment_report(id)
        .await
        .map_err(failure)?;
    if report.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(report)
}

async fn require_task(context: &ActionContext<'_>, id: TaskId) -> Result<(), String> {
    let task = context.usecase.get_task(id).await.map_err(failure)?;
    if task.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(())
}

async fn list_tasks(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let work_date = context
        .optional_text("work_date")?
        .map(|raw| date(raw, "work_date"))
        .transpose()?;
    seat_of(context, member).await?;

    let tasks = context
        .usecase
        .list_tasks_assigned_to_member_on(context.organization_id, member, work_date)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        tasks.iter().map(|task| task_json(task, member)).collect(),
    ))
}

async fn get_current(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let employee = employee_of(context, member).await?;

    let (running, day_log) = context
        .usecase
        .find_field_status(context.organization_id, employee)
        .await
        .map_err(failure)?;

    Ok(json!({
        "running": running.as_ref().map(entry_json),
        "day_ended_at": day_log.map(|log| log.ended_at),
    }))
}

async fn start_time_entry(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let task_id = TaskId(context.uuid("task_id")?);
    let employee_id = employee_of(context, member).await?;
    require_task(context, task_id).await?;

    context
        .usecase
        .start_time_entry(StartTimeEntryCommand {
            organization_id: context.organization_id,
            task_id,
            employee_id,
            at: Utc::now(),
        })
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn declare_time_entry(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let task_id = TaskId(context.uuid("task_id")?);
    let started_at = moment(context, "started_at")?;
    let ended_at = moment(context, "ended_at")?;
    let employee_id = employee_of(context, member).await?;
    require_task(context, task_id).await?;

    context
        .usecase
        .declare_time_entry(DeclareTimeEntryCommand {
            organization_id: context.organization_id,
            task_id,
            employee_id,
            started_at,
            ended_at,
            now: Utc::now(),
        })
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn stop_time_entry(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = TimeEntryId(context.uuid("time_entry_id")?);
    entry_of(context, id).await?;

    context
        .usecase
        .stop_time_entry(context.organization_id, id, Utc::now())
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn recover_time_entry(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = TimeEntryId(context.uuid("time_entry_id")?);
    let ended_at = moment(context, "ended_at")?;
    entry_of(context, id).await?;

    context
        .usecase
        .recover_forgotten_time_entry(context.organization_id, id, ended_at, Utc::now())
        .await
        .map(|entry| entry_json(&entry))
        .map_err(failure)
}

async fn attach_photo(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = TimeEntryId(context.uuid("time_entry_id")?);
    let phase = TimeEntryPhotoPhase::from_str(context.text("phase")?).map_err(failure_text)?;
    let storage_key = context
        .filled("storage_key", MAX_STORAGE_KEY_CHARS)?
        .to_owned();
    entry_of(context, id).await?;

    context
        .usecase
        .attach_time_entry_photo(AttachTimeEntryPhotoCommand {
            time_entry_id: id,
            phase,
            storage_key,
            at: Utc::now(),
        })
        .await
        .map(|photo| photo_json(&photo))
        .map_err(failure)
}

fn failure_text(error: impl ToString) -> String {
    error.to_string()
}

async fn end_day(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let ended_at = optional_moment(context, "ended_at")?.unwrap_or_else(Utc::now);
    let employee = employee_of(context, member).await?;

    context
        .usecase
        .end_day(context.organization_id, employee, ended_at)
        .await
        .map(|log| day_log_json(&log))
        .map_err(failure)
}

async fn report_assignment(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let task_assignment_id = TaskAssignmentId(context.uuid("task_assignment_id")?);
    let reported_minutes = minutes(context)?;
    let comment = comment(context)?;
    seat_of(context, member).await?;

    let report = context
        .usecase
        .report_assignment(ReportAssignmentCommand {
            task_assignment_id,
            reported_by: member,
            reported_minutes,
            comment,
        })
        .await
        .map_err(failure)?;
    if report.organization_id != context.organization_id {
        return Err(not_found());
    }

    Ok(report_json(&report))
}

async fn amend_report(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = AssignmentReportId(context.uuid("assignment_report_id")?);
    let reported_minutes = minutes(context)?;
    let comment = comment(context)?;
    let existing = report_of(context, id).await?;

    context
        .usecase
        .amend_assignment_report(AmendAssignmentReportCommand {
            id,
            acting_member_id: existing.reported_by,
            reported_minutes,
            comment,
        })
        .await
        .map(|report| report_json(&report))
        .map_err(failure)
}

async fn withdraw_report(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = AssignmentReportId(context.uuid("assignment_report_id")?);
    let existing = report_of(context, id).await?;

    context
        .usecase
        .withdraw_assignment_report(WithdrawAssignmentReportCommand {
            id,
            acting_member_id: existing.reported_by,
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "assignment_report_id": id.0, "withdrawn": true }))
}

async fn list_reports(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = context.optional_uuid("member_id")?.map(MemberId);
    let resolution = context
        .optional_text("resolution")?
        .map(|raw| AssignmentReportResolution::from_str(raw).map_err(failure_text))
        .transpose()?;
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);

    let (reports, _) = match member {
        Some(member) => {
            seat_of(context, member).await?;
            context
                .usecase
                .list_assignment_reports_by_reporter(
                    context.organization_id,
                    member,
                    resolution,
                    limit,
                    offset,
                )
                .await
        }
        None => {
            context
                .usecase
                .list_assignment_reports_by_organization(
                    context.organization_id,
                    resolution,
                    limit,
                    offset,
                )
                .await
        }
    }
    .map_err(failure)?;

    Ok(Value::Array(reports.iter().map(report_json).collect()))
}

async fn propose_intervention(context: &ActionContext<'_>) -> Result<Value, String> {
    let member = member_id(context)?;
    let task_id = TaskId(context.uuid("task_id")?);
    let done_on = date(context.text("done_on")?.trim(), "done_on")?;
    let seat = seat_of(context, member).await?;
    let user: UserId = seat
        .user_id
        .ok_or_else(|| "the member has no user account to act with".to_owned())?;
    require_task(context, task_id).await?;

    context
        .usecase
        .propose_maintenance_intervention(ProposeInterventionCommand {
            actor: user_subject(user, Vec::new()),
            organization_id: context.organization_id,
            task_id,
            done_on,
        })
        .await
        .map(|intervention| intervention_json(&intervention))
        .map_err(failure)
}
