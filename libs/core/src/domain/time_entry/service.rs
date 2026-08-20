use chrono::{DateTime, TimeZone, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    DayLog, DayLogId, TimeEntry, TimeEntryId, TimeEntryPhoto, TimeEntryPhotoId, Tz,
    domain::time_entry::{
        commands::{
            AttachTimeEntryPhotoCommand, EndDayCommand, RecoverTimeEntryCommand,
            StartTimeEntryCommand, StopTimeEntryCommand,
        },
        events::{DayEnded, TimeEntryStarted, TimeEntryStopped},
        ports::{DayLogRepository, TimeEntryRepository},
    },
};

pub struct TimeEntryService<R, D, E>
where
    R: TimeEntryRepository,
    D: DayLogRepository,
    E: EventEmitter,
{
    entries: R,
    day_logs: D,
    emitter: E,
}

impl<R, D, E> TimeEntryService<R, D, E>
where
    R: TimeEntryRepository,
    D: DayLogRepository,
    E: EventEmitter,
{
    pub fn new(entries: R, day_logs: D, emitter: E) -> Self {
        Self {
            entries,
            day_logs,
            emitter,
        }
    }

    /// Clocks the employee on to a task.
    ///
    /// Refuses when they are already clocked on somewhere. The database would
    /// refuse too, via a partial unique index, but a constraint violation
    /// cannot tell the caller *which* job is still open, and that is the one
    /// thing the field app needs in order to offer a way out.
    pub async fn start(&mut self, command: StartTimeEntryCommand) -> Result<TimeEntry, CoreError> {
        if let Some(running) = self
            .entries
            .find_running_for_employee(command.employee_id)
            .await?
        {
            return Err(CoreError::Conflict(format!(
                "employee is already clocked on to task {}",
                running.task_id
            )));
        }

        let entry = self
            .entries
            .insert(&TimeEntry {
                id: TimeEntryId(generate_uuid_v7()),
                organization_id: command.organization_id,
                task_id: command.task_id,
                employee_id: command.employee_id,
                started_at: command.at,
                ended_at: None,
                photos: vec![],
                closed_after_the_fact: false,
                created_at: command.at,
                updated_at: command.at,
            })
            .await?;

        self.emitter.emit(
            entry.organization_id,
            &TimeEntryStarted {
                entry: entry.clone(),
            },
        )?;

        Ok(entry)
    }

    /// Closes a running entry.
    ///
    /// An entry already closed is a conflict rather than a no-op: two stops
    /// mean the field app and the server disagree about what is running, and
    /// silently accepting the second would hide that.
    pub async fn stop(&mut self, command: StopTimeEntryCommand) -> Result<TimeEntry, CoreError> {
        let existing = self
            .entries
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        validate_close(&existing, command.at)?;
        refuse_stale(&existing, command.at, command.timezone)?;

        let stopped = self.entries.close(existing.id, command.at, false).await?;

        self.emitter.emit(
            stopped.organization_id,
            &TimeEntryStopped {
                entry: stopped.clone(),
            },
        )?;

        Ok(stopped)
    }

    /// Closes a stretch the employee forgot, at the time they now declare.
    ///
    /// The only path allowed to accept an end on a later day than the start,
    /// because it is the only one where somebody is stating a fact rather than
    /// pressing a button at the moment it happens. The entry is marked, so no
    /// reader mistakes the recollection for a measurement.
    ///
    /// Refuses an entry that is not actually stale: a running job from today is
    /// stopped, not recovered, and letting this route close it would put a
    /// declared time where a measured one belongs.
    pub async fn recover_forgotten(
        &mut self,
        command: RecoverTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        let existing = self
            .entries
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        validate_close(&existing, command.ended_at)?;
        if !existing.is_stale(command.now, command.timezone) {
            return Err(CoreError::Conflict(
                "this stretch is from today, so it is stopped rather than recovered".to_owned(),
            ));
        }

        let stopped = self
            .entries
            .close(existing.id, command.ended_at, true)
            .await?;

        self.emitter.emit(
            stopped.organization_id,
            &TimeEntryStopped {
                entry: stopped.clone(),
            },
        )?;

        Ok(stopped)
    }

    /// Attaches a photo to an entry.
    ///
    /// Allowed while running and after closing: an employee photographing the
    /// finished work often does so after pressing stop, and refusing that
    /// would cost the "after" half of every before/after pair.
    pub async fn attach_photo(
        &mut self,
        command: AttachTimeEntryPhotoCommand,
    ) -> Result<TimeEntryPhoto, CoreError> {
        let entry = self
            .entries
            .find_by_id(command.time_entry_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let storage_key = command.storage_key.trim();
        if storage_key.is_empty() {
            return Err(CoreError::Conflict(
                "photo storage key cannot be empty".to_owned(),
            ));
        }

        self.entries
            .attach_photo(&TimeEntryPhoto {
                id: TimeEntryPhotoId(generate_uuid_v7()),
                organization_id: entry.organization_id,
                time_entry_id: entry.id,
                phase: command.phase,
                storage_key: storage_key.to_owned(),
                created_at: command.at,
            })
            .await
    }

    /// The entry the employee is clocked on to, if any.
    ///
    /// The field app asks on every load: it decides whether the screen offers
    /// "start" or "stop", and getting it wrong is what lets a second job open.
    pub async fn find(&mut self, id: TimeEntryId) -> Result<Option<TimeEntry>, CoreError> {
        self.entries.find_by_id(id).await
    }

    pub async fn running_for(
        &mut self,
        employee_id: crate::EmployeeId,
    ) -> Result<Option<TimeEntry>, CoreError> {
        self.entries.find_running_for_employee(employee_id).await
    }

    /// The day log for today, if the employee already declared it over.
    ///
    /// Thin pass-through, symmetrical with `running_for`: the field app asks
    /// both on every load, and neither needs anything beyond what the
    /// repository already knows.
    pub async fn day_log_for_today(
        &mut self,
        employee_id: crate::EmployeeId,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Option<DayLog>, CoreError> {
        let work_date = local_date(now, timezone);
        self.day_logs
            .find_for_employee_on(employee_id, work_date)
            .await
    }

    pub async fn list_for_employee_on(
        &mut self,
        employee_id: crate::EmployeeId,
        work_date: chrono::NaiveDate,
    ) -> Result<Vec<TimeEntry>, CoreError> {
        self.entries
            .list_for_employee_on(employee_id, work_date)
            .await
    }

    /// Declares the day over, closing whatever the employee left running.
    ///
    /// The closing time is the one they declared, not now: someone filling
    /// this in on the drive home is stating when they stopped, and recording
    /// the later moment would bill the customer for the journey.
    pub async fn end_day(&mut self, command: EndDayCommand) -> Result<DayLog, CoreError> {
        let work_date = local_date(command.ended_at, command.timezone);

        let mut closed_entries = 0;
        if let Some(running) = self
            .entries
            .find_running_for_employee(command.employee_id)
            .await?
        {
            validate_close(&running, command.ended_at)?;
            refuse_stale(&running, command.ended_at, command.timezone)?;
            let stopped = self
                .entries
                .close(running.id, command.ended_at, false)
                .await?;
            closed_entries += 1;

            self.emitter.emit(
                stopped.organization_id,
                &TimeEntryStopped {
                    entry: stopped.clone(),
                },
            )?;
        }

        let day_log = self
            .day_logs
            .upsert(&DayLog {
                id: DayLogId(generate_uuid_v7()),
                organization_id: command.organization_id,
                employee_id: command.employee_id,
                work_date,
                ended_at: command.ended_at,
                created_at: command.ended_at,
            })
            .await?;

        self.emitter.emit(
            day_log.organization_id,
            &DayEnded {
                day_log: day_log.clone(),
                closed_entries,
            },
        )?;

        Ok(day_log)
    }
}

/// The two ways closing an entry can be wrong, in one place so `stop` and
/// `end_day` cannot drift apart.
fn validate_close(entry: &TimeEntry, at: DateTime<Utc>) -> Result<(), CoreError> {
    if !entry.is_running() {
        return Err(CoreError::Conflict(
            "time entry is already closed".to_owned(),
        ));
    }

    if at <= entry.started_at {
        return Err(CoreError::Conflict(
            "a time entry cannot end before it started".to_owned(),
        ));
    }

    Ok(())
}

/// Refuses to close a stretch that began on an earlier local day.
///
/// This is the forgotten clock-off, and closing it at the current time is how a
/// three-hour job became thirty-four: `validate_close` only checks that the end
/// follows the start, which a whole day later satisfies. The employee is asked
/// for the real time instead, through `recover_forgotten`.
fn refuse_stale(entry: &TimeEntry, at: DateTime<Utc>, timezone: Tz) -> Result<(), CoreError> {
    if entry.is_stale(at, timezone) {
        return Err(CoreError::Conflict(
            "this stretch began on an earlier day and needs its end time declared".to_owned(),
        ));
    }

    Ok(())
}

/// The calendar day an instant falls in, for the organization.
///
/// A day ended at 23:30 in Paris is 21:30 UTC and belongs to that day; read
/// as UTC it still would, but one ended at 00:30 local would be filed under
/// the previous day, and the employee's log would show two days' work on one.
fn local_date(at: DateTime<Utc>, timezone: Tz) -> chrono::NaiveDate {
    timezone.from_utc_datetime(&at.naive_utc()).date_naive()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use events::testing::RecordingEmitter;
    use uuid::Uuid;

    use super::*;
    use crate::{
        EmployeeId, OrganizationId, TaskId, TimeEntryPhotoPhase,
        domain::time_entry::ports::{MockDayLogRepository, MockTimeEntryRepository},
    };

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 18, hour, minute, 0)
            .single()
            .expect("a valid test instant")
    }

    fn employee() -> EmployeeId {
        EmployeeId(Uuid::new_v4())
    }

    fn entry(
        id: TimeEntryId,
        employee_id: EmployeeId,
        ended_at: Option<DateTime<Utc>>,
    ) -> TimeEntry {
        TimeEntry {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            task_id: TaskId(Uuid::new_v4()),
            employee_id,
            started_at: at(8, 0),
            ended_at,
            photos: vec![],
            closed_after_the_fact: false,
            created_at: at(8, 0),
            updated_at: at(8, 0),
        }
    }

    fn start_command(employee_id: EmployeeId) -> StartTimeEntryCommand {
        StartTimeEntryCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            task_id: TaskId(Uuid::new_v4()),
            employee_id,
            at: at(8, 0),
        }
    }

    /// Clocking on records the moment the employee gave, not the service's own
    /// clock, which is what lets a stamp survive a slow request.
    #[tokio::test]
    async fn starting_records_the_given_instant_and_leaves_the_entry_running() {
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries
            .expect_find_running_for_employee()
            .returning(|_| Box::pin(async { Ok(None) }));
        entries.expect_insert().returning(|e| {
            let e = e.clone();
            Box::pin(async move { Ok(e) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let started = service.start(start_command(employee_id)).await.unwrap();

        assert_eq!(started.started_at, at(8, 0));
        assert!(started.is_running());
        assert_eq!(started.worked_minutes(), None);
        assert_eq!(emitter.names(), vec!["time_entry.started"]);
    }

    /// The rule the whole field app rests on. The database enforces it too, but
    /// the error has to name the job that is still open.
    #[tokio::test]
    async fn a_second_job_is_refused_while_one_is_running() {
        let employee_id = employee();
        let running = entry(TimeEntryId(Uuid::new_v4()), employee_id, None);
        let open_task = running.task_id;
        let mut entries = MockTimeEntryRepository::new();
        entries
            .expect_find_running_for_employee()
            .returning(move |_| {
                let running = running.clone();
                Box::pin(async move { Ok(Some(running)) })
            });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service.start(start_command(employee_id)).await;

        match outcome {
            Err(CoreError::Conflict(message)) => assert!(
                message.contains(&open_task.to_string()),
                "the refusal must name the open job, got: {message}"
            ),
            other => panic!("expected a conflict naming the open job, got {other:?}"),
        }
        assert!(emitter.names().is_empty(), "a refused start emits nothing");
    }

    #[tokio::test]
    async fn stopping_closes_the_entry_and_reports_the_minutes_worked() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = entry(id, employee_id, None);
            Box::pin(async move { Ok(Some(e)) })
        });
        entries.expect_close().returning(move |_, ended_at, _| {
            let mut e = entry(id, employee_id, Some(ended_at));
            e.updated_at = ended_at;
            Box::pin(async move { Ok(e) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let stopped = service
            .stop(StopTimeEntryCommand {
                id,
                at: at(12, 15),
                timezone: chrono_tz::Europe::Paris,
            })
            .await
            .unwrap();

        assert_eq!(stopped.worked_minutes(), Some(255));
        let payload = emitter.only("time_entry.stopped").payload;
        assert_eq!(payload["worked_minutes"], serde_json::json!(255));
    }

    /// Two stops mean the app and the server disagree about what is running.
    /// Accepting the second silently would hide that.
    #[tokio::test]
    async fn stopping_an_already_closed_entry_is_refused() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = entry(id, employee_id, Some(at(11, 0)));
            Box::pin(async move { Ok(Some(e)) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .stop(StopTimeEntryCommand {
                id,
                at: at(12, 0),
                timezone: chrono_tz::Europe::Paris,
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn an_entry_cannot_end_before_it_started() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = entry(id, employee_id, None);
            Box::pin(async move { Ok(Some(e)) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .stop(StopTimeEntryCommand {
                id,
                at: at(7, 0),
                timezone: chrono_tz::Europe::Paris,
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
    }

    /// Ending the day closes what was left running, at the declared moment
    /// rather than now: someone filling this in on the way home is stating
    /// when they stopped, and the later instant would bill the journey.
    #[tokio::test]
    async fn ending_the_day_closes_the_running_entry_at_the_declared_time() {
        let employee_id = employee();
        let id = TimeEntryId(Uuid::new_v4());
        let mut entries = MockTimeEntryRepository::new();
        entries
            .expect_find_running_for_employee()
            .returning(move |_| {
                let e = entry(id, employee_id, None);
                Box::pin(async move { Ok(Some(e)) })
            });
        entries.expect_close().returning(move |_, ended_at, _| {
            let e = entry(id, employee_id, Some(ended_at));
            Box::pin(async move { Ok(e) })
        });
        let mut day_logs = MockDayLogRepository::new();
        day_logs.expect_upsert().returning(|log| {
            let log = log.clone();
            Box::pin(async move { Ok(log) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, day_logs, &emitter);

        let day_log = service
            .end_day(EndDayCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                employee_id,
                ended_at: at(16, 30),
                timezone: chrono_tz::Europe::Paris,
            })
            .await
            .unwrap();

        assert_eq!(day_log.ended_at, at(16, 30));
        assert_eq!(
            emitter.names(),
            vec!["time_entry.stopped", "day.ended"],
            "the entry closes before the day is declared over"
        );
        assert_eq!(
            emitter.only("day.ended").payload["closed_entries"],
            serde_json::json!(1)
        );
    }

    #[tokio::test]
    async fn ending_a_day_with_nothing_running_closes_nothing() {
        let mut entries = MockTimeEntryRepository::new();
        entries
            .expect_find_running_for_employee()
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut day_logs = MockDayLogRepository::new();
        day_logs.expect_upsert().returning(|log| {
            let log = log.clone();
            Box::pin(async move { Ok(log) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, day_logs, &emitter);

        service
            .end_day(EndDayCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                employee_id: employee(),
                ended_at: at(16, 30),
                timezone: chrono_tz::Europe::Paris,
            })
            .await
            .unwrap();

        assert_eq!(emitter.names(), vec!["day.ended"]);
        assert_eq!(
            emitter.only("day.ended").payload["closed_entries"],
            serde_json::json!(0)
        );
    }

    /// The reason the timezone is a parameter. 23:30 in Paris is 21:30 UTC on
    /// the same day, but 00:30 in Paris is 22:30 UTC on the *previous* one, and
    /// filing that under the previous date would show two days' work on one.
    #[tokio::test]
    async fn the_work_date_follows_the_organization_timezone_not_utc() {
        let just_past_midnight_in_paris = Utc
            .with_ymd_and_hms(2026, 8, 18, 22, 30, 0)
            .single()
            .expect("a valid test instant");
        let mut entries = MockTimeEntryRepository::new();
        entries
            .expect_find_running_for_employee()
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut day_logs = MockDayLogRepository::new();
        day_logs.expect_upsert().returning(|log| {
            let log = log.clone();
            Box::pin(async move { Ok(log) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, day_logs, &emitter);

        let day_log = service
            .end_day(EndDayCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                employee_id: employee(),
                ended_at: just_past_midnight_in_paris,
                timezone: chrono_tz::Europe::Paris,
            })
            .await
            .unwrap();

        assert_eq!(
            day_log.work_date,
            chrono::NaiveDate::from_ymd_opt(2026, 8, 19).expect("a valid date"),
            "22:30 UTC is 00:30 the next day in Paris"
        );
    }

    /// Yesterday's instant, so an entry can be made stale without waiting a day.
    fn yesterday(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 17, hour, 0, 0)
            .single()
            .expect("a valid test instant")
    }

    const PARIS: chrono_tz::Tz = chrono_tz::Europe::Paris;

    fn stale_entry(id: TimeEntryId, employee_id: EmployeeId) -> TimeEntry {
        let mut entry = entry(id, employee_id, None);
        entry.started_at = yesterday(8);
        entry
    }

    /// The bug this whole change exists for. Closing a stretch begun yesterday
    /// at today's time turned three hours of work into thirty-four, and
    /// `validate_close` waved it through because the end does follow the start.
    #[tokio::test]
    async fn stopping_a_stretch_begun_yesterday_is_refused_rather_than_inflated() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = stale_entry(id, employee_id);
            Box::pin(async move { Ok(Some(e)) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .stop(StopTimeEntryCommand {
                id,
                at: at(18, 0),
                timezone: PARIS,
            })
            .await;

        assert!(
            matches!(outcome, Err(CoreError::Conflict(_))),
            "{outcome:?}"
        );
        assert!(emitter.names().is_empty(), "a refused stop emits nothing");
    }

    /// The same hole, reached through the other door: the field app's "end my
    /// day" button used to close yesterday's forgotten stretch at this evening's
    /// time, which is where the phantom hours actually came from.
    #[tokio::test]
    async fn ending_the_day_will_not_close_a_stretch_begun_yesterday() {
        let employee_id = employee();
        let id = TimeEntryId(Uuid::new_v4());
        let mut entries = MockTimeEntryRepository::new();
        entries
            .expect_find_running_for_employee()
            .returning(move |_| {
                let e = stale_entry(id, employee_id);
                Box::pin(async move { Ok(Some(e)) })
            });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .end_day(EndDayCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                employee_id,
                ended_at: at(18, 0),
                timezone: PARIS,
            })
            .await;

        assert!(
            matches!(outcome, Err(CoreError::Conflict(_))),
            "{outcome:?}"
        );
        assert!(
            emitter.names().is_empty(),
            "no day is declared over while a forgotten stretch is open"
        );
    }

    #[tokio::test]
    async fn recovering_a_forgotten_stretch_records_the_declared_end_and_marks_it() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = stale_entry(id, employee_id);
            Box::pin(async move { Ok(Some(e)) })
        });
        entries
            .expect_close()
            .returning(move |_, ended_at, after_the_fact| {
                let mut e = stale_entry(id, employee_id);
                e.ended_at = Some(ended_at);
                e.closed_after_the_fact = after_the_fact;
                Box::pin(async move { Ok(e) })
            });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let recovered = service
            .recover_forgotten(RecoverTimeEntryCommand {
                id,
                ended_at: yesterday(17),
                now: at(7, 30),
                timezone: PARIS,
            })
            .await
            .unwrap();

        assert_eq!(recovered.ended_at, Some(yesterday(17)));
        assert_eq!(
            recovered.worked_minutes(),
            Some(540),
            "nine hours, as declared"
        );
        assert!(
            recovered.closed_after_the_fact,
            "a recollection must not read as a measurement"
        );
        assert_eq!(emitter.names(), vec!["time_entry.stopped"]);
    }

    /// A running job from today is stopped, not recovered. Allowing this route
    /// to close it would put a declared time where a measured one belongs, and
    /// mark an honest entry as reconstructed.
    #[tokio::test]
    async fn a_stretch_from_today_cannot_be_recovered() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = entry(id, employee_id, None);
            Box::pin(async move { Ok(Some(e)) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .recover_forgotten(RecoverTimeEntryCommand {
                id,
                ended_at: at(12, 0),
                now: at(14, 0),
                timezone: PARIS,
            })
            .await;

        assert!(
            matches!(outcome, Err(CoreError::Conflict(_))),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_declared_end_before_the_start_is_still_refused() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = stale_entry(id, employee_id);
            Box::pin(async move { Ok(Some(e)) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .recover_forgotten(RecoverTimeEntryCommand {
                id,
                ended_at: yesterday(7),
                now: at(7, 30),
                timezone: PARIS,
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
    }

    /// Staleness is a local-day question, so it follows the organization's
    /// timezone like every other day boundary in this module.
    #[tokio::test]
    async fn staleness_is_judged_in_the_organization_timezone() {
        let entry = stale_entry(TimeEntryId(Uuid::new_v4()), employee());

        // 2026-08-17 08:00 UTC is the 17th in Paris, and `now` is the 18th.
        assert!(entry.is_stale(at(7, 30), PARIS));
        // Still the same stretch, judged against a moment on its own day.
        assert!(!entry.is_stale(yesterday(20), PARIS));
    }

    #[tokio::test]
    async fn a_photo_records_the_phase_it_was_taken_in() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = entry(id, employee_id, None);
            Box::pin(async move { Ok(Some(e)) })
        });
        entries.expect_attach_photo().returning(|photo| {
            let photo = photo.clone();
            Box::pin(async move { Ok(photo) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let photo = service
            .attach_photo(AttachTimeEntryPhotoCommand {
                time_entry_id: id,
                phase: TimeEntryPhotoPhase::After,
                storage_key: "  uploads/field/photo-1.jpg  ".to_owned(),
                at: at(12, 20),
            })
            .await
            .unwrap();

        assert_eq!(photo.phase, TimeEntryPhotoPhase::After);
        assert_eq!(photo.storage_key, "uploads/field/photo-1.jpg");
    }

    #[tokio::test]
    async fn a_photo_on_a_closed_entry_is_still_accepted() {
        let id = TimeEntryId(Uuid::new_v4());
        let employee_id = employee();
        let mut entries = MockTimeEntryRepository::new();
        entries.expect_find_by_id().returning(move |_| {
            let e = entry(id, employee_id, Some(at(12, 0)));
            Box::pin(async move { Ok(Some(e)) })
        });
        entries.expect_attach_photo().returning(|photo| {
            let photo = photo.clone();
            Box::pin(async move { Ok(photo) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TimeEntryService::new(entries, MockDayLogRepository::new(), &emitter);

        let outcome = service
            .attach_photo(AttachTimeEntryPhotoCommand {
                time_entry_id: id,
                phase: TimeEntryPhotoPhase::After,
                storage_key: "uploads/field/after.jpg".to_owned(),
                at: at(12, 20),
            })
            .await;

        assert!(
            outcome.is_ok(),
            "the after photo is usually taken once the job is stopped"
        );
    }
}
