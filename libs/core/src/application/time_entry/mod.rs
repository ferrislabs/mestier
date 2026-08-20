use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DayLog, EmployeeId, MemberId, OrganizationId, Task, TimeEntry, TimeEntryId, TimeEntryPhoto,
    application::{MestierUseCase, organization::resolve_timezone},
    domain::task::service::TaskService,
    domain::time_entry::{
        commands::{
            AttachTimeEntryPhotoCommand, DeclareTimeEntryCommand, EndDayCommand,
            RecoverTimeEntryCommand, StartTimeEntryCommand, StopTimeEntryCommand,
        },
        service::TimeEntryService,
    },
};

impl MestierUseCase {
    #[transactional(time_entry, day_log, emitter)]
    pub async fn start_time_entry(
        &self,
        command: StartTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);
        service.start(command).await
    }

    /// Resolves the timezone here: whether a stretch is a normal clock-off or a
    /// forgotten one is a question about calendar days, and the HTTP layer has
    /// no business answering it.
    #[transactional(time_entry, day_log, organization, emitter)]
    pub async fn stop_time_entry(
        &self,
        organization_id: OrganizationId,
        id: TimeEntryId,
        at: DateTime<Utc>,
    ) -> Result<TimeEntry, CoreError> {
        let mut organization_repository = organization_repository;
        let timezone = resolve_timezone(&mut organization_repository, organization_id).await?;
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);

        service
            .stop(StopTimeEntryCommand { id, at, timezone })
            .await
    }

    /// Declares a stretch of work that was never clocked live at all — the
    /// rush that left no time to press "start". Server-stamps `now` for the
    /// same reason `start_time_entry` server-stamps `at`: a client clock is
    /// trusted only once the offline mode exists to justify it.
    #[transactional(time_entry, day_log, emitter)]
    pub async fn declare_time_entry(
        &self,
        command: DeclareTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);
        service.declare(command).await
    }

    /// Closes a stretch the employee forgot, at the time they now declare.
    #[transactional(time_entry, day_log, organization, emitter)]
    pub async fn recover_forgotten_time_entry(
        &self,
        organization_id: OrganizationId,
        id: TimeEntryId,
        ended_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<TimeEntry, CoreError> {
        let mut organization_repository = organization_repository;
        let timezone = resolve_timezone(&mut organization_repository, organization_id).await?;
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);

        service
            .recover_forgotten(RecoverTimeEntryCommand {
                id,
                ended_at,
                now,
                timezone,
            })
            .await
    }

    #[transactional(time_entry, day_log, emitter)]
    pub async fn attach_time_entry_photo(
        &self,
        command: AttachTimeEntryPhotoCommand,
    ) -> Result<TimeEntryPhoto, CoreError> {
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);
        service.attach_photo(command).await
    }

    /// Resolves the organization's timezone here rather than taking it from
    /// the caller: it decides which calendar day the declaration lands on, and
    /// an HTTP layer that had to supply it could supply the wrong one.
    #[transactional(time_entry, day_log, organization, emitter)]
    pub async fn end_day(
        &self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        ended_at: DateTime<Utc>,
    ) -> Result<DayLog, CoreError> {
        let mut organization_repository = organization_repository;
        let timezone = resolve_timezone(&mut organization_repository, organization_id).await?;
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);

        service
            .end_day(EndDayCommand {
                organization_id,
                employee_id,
                ended_at,
                timezone,
            })
            .await
    }

    /// The entry the employee is currently clocked on to, if any.
    ///
    /// The field app asks on every load: it is what decides whether the screen
    /// offers "start" or "stop", and getting it wrong is what lets someone
    /// open a second job.
    /// One entry by id, for the routes that must check ownership before acting.
    #[transactional(time_entry, day_log, emitter)]
    pub async fn get_time_entry(&self, id: TimeEntryId) -> Result<Option<TimeEntry>, CoreError> {
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);
        service.find(id).await
    }

    #[transactional(time_entry, day_log, emitter)]
    pub async fn find_running_time_entry(
        &self,
        employee_id: EmployeeId,
    ) -> Result<Option<TimeEntry>, CoreError> {
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);
        service.running_for(employee_id).await
    }

    /// What the field app needs on load: the running entry, if any, and the
    /// day log for today, if the employee already declared the day over.
    ///
    /// Resolves the organization's timezone the same way `profitability_report`
    /// does, because deciding what "today" means is the same question in both
    /// places, and answering it differently here would file the day boundary
    /// somewhere the rest of the system does not agree with.
    #[transactional(time_entry, day_log, organization, emitter)]
    pub async fn find_field_status(
        &self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
    ) -> Result<(Option<TimeEntry>, Option<DayLog>), CoreError> {
        let mut organization_repository = organization_repository;
        let timezone = resolve_timezone(&mut organization_repository, organization_id).await?;
        let now = Utc::now();
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);

        let running = service.running_for(employee_id).await?;
        let day_log = service
            .day_log_for_today(employee_id, now, timezone)
            .await?;

        Ok((running, day_log))
    }

    #[transactional(time_entry, day_log, emitter)]
    pub async fn list_time_entries_for_employee_on(
        &self,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> Result<Vec<TimeEntry>, CoreError> {
        let mut service = TimeEntryService::new(time_entry_repository, day_log_repository, emitter);
        service.list_for_employee_on(employee_id, work_date).await
    }

    /// "Mes chantiers" for one day, in the organization's timezone.
    ///
    /// The day is resolved here rather than by the caller so that every reader
    /// agrees on where a day starts, and so the HTTP layer never has to know
    /// the organization's timezone to ask a simple question.
    /// `work_date` defaults to today **in the organization's timezone**, not in
    /// UTC. Those differ for part of every day, and a worker on a late shift
    /// asking for "today" would otherwise be handed yesterday's jobs.
    #[transactional(task, member, organization)]
    pub async fn list_tasks_assigned_to_member_on(
        &self,
        organization_id: OrganizationId,
        member_id: MemberId,
        work_date: Option<NaiveDate>,
    ) -> Result<Vec<Task>, CoreError> {
        let mut organization_repository = organization_repository;
        let timezone = resolve_timezone(&mut organization_repository, organization_id).await?;
        let work_date =
            work_date.unwrap_or_else(|| Utc::now().with_timezone(&timezone).date_naive());
        let (starts_at, ends_at) = day_bounds(work_date, timezone)?;
        let mut service = TaskService::new(task_repository, member_repository);

        service
            .list_assigned_to_member_on(organization_id, member_id, starts_at, ends_at)
            .await
    }
}

/// The instants a local calendar day starts and ends at.
///
/// Uses the *earliest* valid local midnight: on a spring-forward night the
/// wall clock skips an hour, and `single()` would find nothing. Failing there
/// would make the field app unusable one morning a year.
fn day_bounds(
    work_date: NaiveDate,
    timezone: crate::Tz,
) -> Result<(DateTime<Utc>, DateTime<Utc>), CoreError> {
    let start_local = work_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| CoreError::Internal("midnight is a valid time on every date".to_owned()))?;
    let next_local = work_date
        .succ_opt()
        .and_then(|next| next.and_hms_opt(0, 0, 0))
        .ok_or_else(|| CoreError::Internal("the day after has a midnight".to_owned()))?;

    let starts_at = timezone
        .from_local_datetime(&start_local)
        .earliest()
        .ok_or_else(|| CoreError::Internal("no valid local start of day".to_owned()))?
        .with_timezone(&Utc);
    let ends_at = timezone
        .from_local_datetime(&next_local)
        .earliest()
        .ok_or_else(|| CoreError::Internal("no valid local end of day".to_owned()))?
        .with_timezone(&Utc);

    Ok((starts_at, ends_at))
}
