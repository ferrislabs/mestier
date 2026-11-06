use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use common::OrganizationId;
use uuid::Uuid;

use super::graph::{Frequency, PlacedTrigger, TriggerKind, Weekday};

pub type Instant = DateTime<Utc>;

const SEARCH_DAYS: i64 = 10;
const SEARCH_MINUTES: i64 = 180;
const GAP_SEARCH_MINUTES: i64 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleRule {
    Hourly {
        minute: u32,
    },
    Daily {
        hour: u32,
        minute: u32,
    },
    Weekly {
        weekday: Weekday,
        hour: u32,
        minute: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ScheduleRuleError {
    #[error("the time must be written HH:MM, from 00:00 to 23:59")]
    InvalidTime,
    #[error("a weekly schedule needs a weekday")]
    MissingWeekday,
}

pub fn parse_time(value: &str) -> Option<(u32, u32)> {
    let bytes = value.as_bytes();
    let well_formed = bytes.len() == 5
        && bytes[2] == b':'
        && [0, 1, 3, 4]
            .iter()
            .all(|&index| bytes[index].is_ascii_digit());
    if !well_formed {
        return None;
    }
    let hour: u32 = value[..2].parse().ok()?;
    let minute: u32 = value[3..].parse().ok()?;
    (hour < 24 && minute < 60).then_some((hour, minute))
}

pub fn parse_timezone(name: &str) -> Option<Tz> {
    name.parse::<Tz>().ok()
}

impl ScheduleRule {
    pub fn new(
        frequency: Frequency,
        time: &str,
        weekday: Option<Weekday>,
    ) -> Result<Self, ScheduleRuleError> {
        let (hour, minute) = parse_time(time).ok_or(ScheduleRuleError::InvalidTime)?;
        match frequency {
            Frequency::Hourly => Ok(Self::Hourly { minute }),
            Frequency::Daily => Ok(Self::Daily { hour, minute }),
            Frequency::Weekly => {
                let weekday = weekday.ok_or(ScheduleRuleError::MissingWeekday)?;
                Ok(Self::Weekly {
                    weekday,
                    hour,
                    minute,
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleSeed {
    pub trigger_id: String,
    pub next_run_at: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DueSchedule {
    pub org_id: OrganizationId,
    pub workflow_id: Uuid,
    pub workflow_version_id: Uuid,
    pub trigger_id: String,
    pub scheduled_for: Instant,
    pub trigger: TriggerKind,
    pub organization_timezone: String,
}

pub fn recurrence_of(kind: &TriggerKind, organization_zone: Tz) -> Option<(ScheduleRule, Tz)> {
    let TriggerKind::Schedule {
        frequency,
        time,
        weekday,
        timezone,
    } = kind
    else {
        return None;
    };
    let rule = ScheduleRule::new(*frequency, time, *weekday).ok()?;
    let zone = match timezone {
        Some(name) => parse_timezone(name)?,
        None => organization_zone,
    };
    Some((rule, zone))
}

pub fn schedule_seeds(
    triggers: &[PlacedTrigger],
    organization_zone: Tz,
    now: Instant,
) -> Vec<ScheduleSeed> {
    triggers
        .iter()
        .filter_map(|trigger| {
            let (rule, zone) = recurrence_of(&trigger.kind, organization_zone)?;
            Some(ScheduleSeed {
                trigger_id: trigger.id.clone(),
                next_run_at: next_due(now, &rule, zone),
            })
        })
        .collect()
}

impl DueSchedule {
    pub fn next_after(&self, now: Instant) -> Option<Instant> {
        let organization_zone = parse_timezone(&self.organization_timezone);
        let (rule, zone) = match (&self.trigger, organization_zone) {
            (
                TriggerKind::Schedule {
                    timezone: Some(_), ..
                },
                _,
            ) => recurrence_of(&self.trigger, chrono_tz::UTC)?,
            (_, Some(zone)) => recurrence_of(&self.trigger, zone)?,
            (_, None) => return None,
        };
        Some(next_due(now, &rule, zone))
    }
}

pub fn next_due(after: Instant, rule: &ScheduleRule, timezone: Tz) -> Instant {
    match *rule {
        ScheduleRule::Hourly { minute } => next_hourly(after, minute, timezone),
        ScheduleRule::Daily { hour, minute } => {
            next_on_matching_day(after, hour, minute, timezone, |_| true)
        }
        ScheduleRule::Weekly {
            weekday,
            hour,
            minute,
        } => next_on_matching_day(after, hour, minute, timezone, |date| {
            date.weekday() == chrono_weekday(weekday)
        }),
    }
}

fn chrono_weekday(weekday: Weekday) -> chrono::Weekday {
    match weekday {
        Weekday::Monday => chrono::Weekday::Mon,
        Weekday::Tuesday => chrono::Weekday::Tue,
        Weekday::Wednesday => chrono::Weekday::Wed,
        Weekday::Thursday => chrono::Weekday::Thu,
        Weekday::Friday => chrono::Weekday::Fri,
        Weekday::Saturday => chrono::Weekday::Sat,
        Weekday::Sunday => chrono::Weekday::Sun,
    }
}

fn next_hourly(after: Instant, minute: u32, timezone: Tz) -> Instant {
    let whole_minute = after
        .with_second(0)
        .and_then(|instant| instant.with_nanosecond(0))
        .unwrap_or(after);
    let mut candidate = whole_minute + Duration::minutes(1);
    for _ in 0..SEARCH_MINUTES {
        if candidate.with_timezone(&timezone).minute() == minute {
            return candidate;
        }
        candidate += Duration::minutes(1);
    }
    after + Duration::hours(1)
}

fn next_on_matching_day(
    after: Instant,
    hour: u32,
    minute: u32,
    timezone: Tz,
    matches: impl Fn(NaiveDate) -> bool,
) -> Instant {
    let first_day = after
        .with_timezone(&timezone)
        .date_naive()
        .pred_opt()
        .unwrap_or_else(|| after.date_naive());
    first_day
        .iter_days()
        .take(SEARCH_DAYS as usize)
        .filter(|date| matches(*date))
        .filter_map(|date| local_instant(timezone, date, hour, minute))
        .find(|candidate| *candidate > after)
        .unwrap_or(after + Duration::days(SEARCH_DAYS))
}

fn local_instant(timezone: Tz, date: NaiveDate, hour: u32, minute: u32) -> Option<Instant> {
    let wanted = date.and_hms_opt(hour, minute, 0)?;
    match timezone.from_local_datetime(&wanted) {
        LocalResult::Single(instant) => Some(instant.with_timezone(&Utc)),
        LocalResult::Ambiguous(first, _) => Some(first.with_timezone(&Utc)),
        LocalResult::None => (1..=GAP_SEARCH_MINUTES).find_map(|skipped| {
            match timezone.from_local_datetime(&(wanted + Duration::minutes(skipped))) {
                LocalResult::Single(instant) => Some(instant.with_timezone(&Utc)),
                LocalResult::Ambiguous(first, _) => Some(first.with_timezone(&Utc)),
                LocalResult::None => None,
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> Instant {
        DateTime::parse_from_rfc3339(text)
            .expect("a valid instant")
            .with_timezone(&Utc)
    }

    fn paris() -> Tz {
        chrono_tz::Europe::Paris
    }

    fn daily(time: &str) -> ScheduleRule {
        ScheduleRule::new(Frequency::Daily, time, None).expect("a valid rule")
    }

    fn weekly(weekday: Weekday, time: &str) -> ScheduleRule {
        ScheduleRule::new(Frequency::Weekly, time, Some(weekday)).expect("a valid rule")
    }

    fn hourly(minute: u32) -> ScheduleRule {
        ScheduleRule::new(Frequency::Hourly, &format!("00:{minute:02}"), None)
            .expect("a valid rule")
    }

    #[test]
    fn a_time_is_two_digit_hours_and_minutes() {
        assert_eq!(parse_time("09:00"), Some((9, 0)));
        assert_eq!(parse_time("00:00"), Some((0, 0)));
        assert_eq!(parse_time("23:59"), Some((23, 59)));
    }

    #[test]
    fn anything_else_is_not_a_time() {
        for text in [
            "",
            "9:00",
            "09:0",
            "0900",
            "24:00",
            "09:60",
            " 09:00",
            "09:00 ",
            "09:00:00",
            "ab:cd",
            "-1:00",
            "+9:00",
            "09-00",
            "９:００",
        ] {
            assert_eq!(parse_time(text), None, "`{text}`");
        }
    }

    #[test]
    fn a_timezone_is_an_iana_name() {
        assert_eq!(parse_timezone("Europe/Paris"), Some(paris()));
        assert_eq!(parse_timezone("UTC"), Some(chrono_tz::UTC));
        assert_eq!(parse_timezone("Mars/Olympus"), None);
        assert_eq!(parse_timezone(""), None);
        assert_eq!(parse_timezone(" Europe/Paris"), None);
    }

    #[test]
    fn a_rule_keeps_what_its_frequency_needs() {
        assert_eq!(
            ScheduleRule::new(Frequency::Hourly, "09:15", None),
            Ok(ScheduleRule::Hourly { minute: 15 })
        );
        assert_eq!(
            ScheduleRule::new(Frequency::Daily, "09:15", Some(Weekday::Friday)),
            Ok(ScheduleRule::Daily {
                hour: 9,
                minute: 15
            })
        );
        assert_eq!(
            ScheduleRule::new(Frequency::Weekly, "09:15", Some(Weekday::Friday)),
            Ok(ScheduleRule::Weekly {
                weekday: Weekday::Friday,
                hour: 9,
                minute: 15
            })
        );
    }

    #[test]
    fn a_weekly_rule_without_a_weekday_is_refused() {
        assert_eq!(
            ScheduleRule::new(Frequency::Weekly, "09:00", None),
            Err(ScheduleRuleError::MissingWeekday)
        );
    }

    #[test]
    fn a_rule_with_a_bad_time_is_refused_whatever_its_frequency() {
        for frequency in [Frequency::Hourly, Frequency::Daily, Frequency::Weekly] {
            assert_eq!(
                ScheduleRule::new(frequency, "25:00", Some(Weekday::Monday)),
                Err(ScheduleRuleError::InvalidTime)
            );
        }
    }

    #[test]
    fn a_daily_rule_fires_at_the_local_time_of_the_day() {
        let next = next_due(at("2026-11-06T07:00:00Z"), &daily("09:00"), paris());

        assert_eq!(next, at("2026-11-06T08:00:00Z"));
    }

    #[test]
    fn a_daily_rule_whose_time_has_passed_fires_tomorrow() {
        let next = next_due(at("2026-11-06T09:00:00Z"), &daily("09:00"), paris());

        assert_eq!(next, at("2026-11-07T08:00:00Z"));
    }

    #[test]
    fn the_next_due_time_is_strictly_after_the_given_one() {
        let due = at("2026-11-06T08:00:00Z");

        assert_eq!(
            next_due(due, &daily("09:00"), paris()),
            at("2026-11-07T08:00:00Z")
        );
        assert_eq!(
            next_due(due - Duration::seconds(1), &daily("09:00"), paris()),
            due
        );
        assert_eq!(
            next_due(due + Duration::nanoseconds(1), &daily("09:00"), paris()),
            at("2026-11-07T08:00:00Z")
        );
    }

    #[test]
    fn a_daily_rule_keeps_the_local_time_when_the_clocks_go_forward() {
        let before = at("2026-03-28T08:00:00Z");

        let next = next_due(before, &daily("09:00"), paris());

        assert_eq!(next, at("2026-03-29T07:00:00Z"));
        assert_eq!(next - before, Duration::hours(23));
    }

    #[test]
    fn a_daily_rule_keeps_the_local_time_when_the_clocks_go_back() {
        let before = at("2026-10-24T07:00:00Z");

        let next = next_due(before, &daily("09:00"), paris());

        assert_eq!(next, at("2026-10-25T08:00:00Z"));
        assert_eq!(next - before, Duration::hours(25));
    }

    #[test]
    fn a_time_that_does_not_exist_fires_when_the_clocks_resume() {
        let next = next_due(at("2026-03-28T01:30:00Z"), &daily("02:30"), paris());

        assert_eq!(next, at("2026-03-29T01:00:00Z"));
        assert_eq!(
            next_due(next, &daily("02:30"), paris()),
            at("2026-03-30T00:30:00Z")
        );
    }

    #[test]
    fn a_time_that_happens_twice_fires_on_its_first_occurrence_only() {
        let first = next_due(at("2026-10-24T00:30:00Z"), &daily("02:30"), paris());

        assert_eq!(first, at("2026-10-25T00:30:00Z"));
        assert_eq!(
            next_due(first, &daily("02:30"), paris()),
            at("2026-10-26T01:30:00Z")
        );
    }

    #[test]
    fn a_daily_rule_rolls_over_the_end_of_a_month_and_a_year() {
        let utc = chrono_tz::UTC;
        let rule = daily("09:00");

        for (from, to) in [
            ("2026-01-31T09:00:00Z", "2026-02-01T09:00:00Z"),
            ("2026-04-30T09:00:00Z", "2026-05-01T09:00:00Z"),
            ("2027-02-28T09:00:00Z", "2027-03-01T09:00:00Z"),
            ("2028-02-28T09:00:00Z", "2028-02-29T09:00:00Z"),
            ("2028-02-29T09:00:00Z", "2028-03-01T09:00:00Z"),
            ("2026-12-31T09:00:00Z", "2027-01-01T09:00:00Z"),
        ] {
            assert_eq!(next_due(at(from), &rule, utc), at(to), "from {from}");
        }
    }

    #[test]
    fn a_weekly_rule_fires_on_the_requested_weekday() {
        let rule = weekly(Weekday::Monday, "09:00");

        assert_eq!(
            next_due(at("2026-11-06T12:00:00Z"), &rule, paris()),
            at("2026-11-09T08:00:00Z")
        );
        assert_eq!(
            next_due(at("2026-11-09T07:59:59Z"), &rule, paris()),
            at("2026-11-09T08:00:00Z")
        );
        assert_eq!(
            next_due(at("2026-11-09T08:00:00Z"), &rule, paris()),
            at("2026-11-16T08:00:00Z")
        );
    }

    #[test]
    fn a_weekly_rule_keeps_the_local_time_across_a_clock_change() {
        let rule = weekly(Weekday::Monday, "09:00");

        assert_eq!(
            next_due(at("2026-03-23T08:00:00Z"), &rule, paris()),
            at("2026-03-30T07:00:00Z")
        );
        assert_eq!(
            next_due(at("2026-10-19T07:00:00Z"), &rule, paris()),
            at("2026-10-26T08:00:00Z")
        );
    }

    #[test]
    fn every_weekday_is_reachable() {
        let monday = at("2026-11-09T00:00:00Z");
        let weekdays = [
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ];

        for (offset, weekday) in weekdays.into_iter().enumerate() {
            let next = next_due(
                monday - Duration::seconds(1),
                &weekly(weekday, "00:00"),
                chrono_tz::UTC,
            );

            assert_eq!(next, monday + Duration::days(offset as i64), "{weekday:?}");
        }
    }

    #[test]
    fn a_weekly_rule_crossing_a_month_end_lands_on_the_right_day() {
        let rule = weekly(Weekday::Friday, "09:00");

        assert_eq!(
            next_due(at("2026-10-30T09:00:00Z"), &rule, chrono_tz::UTC),
            at("2026-11-06T09:00:00Z")
        );
        assert_eq!(
            next_due(at("2028-02-25T09:00:00Z"), &rule, chrono_tz::UTC),
            at("2028-03-03T09:00:00Z")
        );
    }

    #[test]
    fn an_hourly_rule_fires_at_its_minute_of_every_hour() {
        let rule = hourly(15);

        assert_eq!(
            next_due(at("2026-11-06T10:20:00Z"), &rule, chrono_tz::UTC),
            at("2026-11-06T11:15:00Z")
        );
        assert_eq!(
            next_due(at("2026-11-06T10:14:59Z"), &rule, chrono_tz::UTC),
            at("2026-11-06T10:15:00Z")
        );
        assert_eq!(
            next_due(at("2026-11-06T10:15:00Z"), &rule, chrono_tz::UTC),
            at("2026-11-06T11:15:00Z")
        );
        assert_eq!(
            next_due(at("2026-11-06T23:15:00Z"), &rule, chrono_tz::UTC),
            at("2026-11-07T00:15:00Z")
        );
    }

    #[test]
    fn an_hourly_rule_reads_its_minute_on_the_local_clock() {
        let kolkata: Tz = chrono_tz::Asia::Kolkata;

        assert_eq!(
            next_due(at("2026-11-06T10:00:00Z"), &hourly(0), kolkata),
            at("2026-11-06T10:30:00Z")
        );
    }

    #[test]
    fn an_hourly_rule_keeps_firing_every_hour_across_a_clock_change() {
        let rule = hourly(0);
        let mut due = at("2026-03-29T00:00:00Z");

        for expected in ["01:00", "02:00", "03:00", "04:00"] {
            due = next_due(due, &rule, paris());
            assert_eq!(due, at(&format!("2026-03-29T{expected}:00Z")));
        }

        let mut due = at("2026-10-25T00:00:00Z");
        for expected in ["01:00", "02:00", "03:00"] {
            due = next_due(due, &rule, paris());
            assert_eq!(due, at(&format!("2026-10-25T{expected}:00Z")));
        }
    }

    fn local_dates_fired(
        rule: &ScheduleRule,
        timezone: Tz,
        from: Instant,
        to: Instant,
    ) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let mut due = next_due(from, rule, timezone);
        while due < to {
            dates.push(due.with_timezone(&timezone).date_naive());
            due = next_due(due, rule, timezone);
        }
        dates
    }

    #[test]
    fn a_daily_rule_fires_exactly_once_per_local_day_over_a_whole_year() {
        let from = at("2025-12-31T23:00:00Z");
        let to = at("2026-12-31T23:00:00Z");

        for time in ["00:30", "02:30", "03:00", "09:00", "23:59"] {
            let dates = local_dates_fired(&daily(time), paris(), from, to);

            let mut expected = Vec::new();
            let mut day = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            while day.year() == 2026 {
                expected.push(day);
                day = day.succ_opt().unwrap();
            }
            assert_eq!(dates, expected, "daily at {time}");
        }
    }

    #[test]
    fn a_weekly_rule_fires_exactly_once_per_week_on_its_weekday() {
        let from = at("2025-12-31T23:00:00Z");
        let to = at("2026-12-31T23:00:00Z");

        let dates = local_dates_fired(&weekly(Weekday::Sunday, "02:30"), paris(), from, to);

        assert_eq!(dates.len(), 52);
        assert!(
            dates
                .iter()
                .all(|date| date.weekday() == chrono::Weekday::Sun)
        );
        assert!(
            dates
                .windows(2)
                .all(|pair| pair[1] - pair[0] == Duration::days(7))
        );
    }

    #[test]
    fn the_next_due_time_is_always_in_the_future_and_never_far() {
        let zones = [
            paris(),
            chrono_tz::UTC,
            chrono_tz::America::New_York,
            chrono_tz::Asia::Kolkata,
            chrono_tz::Australia::Lord_Howe,
            chrono_tz::Pacific::Apia,
        ];
        let rules = [
            hourly(0),
            hourly(59),
            daily("00:00"),
            daily("02:30"),
            daily("23:59"),
            weekly(Weekday::Sunday, "02:30"),
        ];
        let start = at("2026-01-01T00:00:00Z");

        for zone in zones {
            for rule in &rules {
                let mut instant = start;
                for _ in 0..2000 {
                    let next = next_due(instant, rule, zone);
                    assert!(next > instant, "{rule:?} in {zone}: {next} after {instant}");
                    assert!(
                        next - instant <= Duration::days(8),
                        "{rule:?} in {zone}: {next} after {instant}"
                    );
                    instant += Duration::hours(7) + Duration::minutes(13);
                }
            }
        }
    }

    fn schedule_trigger(id: &str, kind: TriggerKind) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind,
            name: None,
            description: None,
        }
    }

    fn daily_kind(time: &str, timezone: Option<&str>) -> TriggerKind {
        TriggerKind::Schedule {
            frequency: Frequency::Daily,
            time: time.to_string(),
            weekday: None,
            timezone: timezone.map(str::to_string),
        }
    }

    #[test]
    fn a_seed_is_the_next_due_time_of_each_schedule_trigger_only() {
        let triggers = vec![
            schedule_trigger("t1", TriggerKind::Manual),
            schedule_trigger(
                "t2",
                TriggerKind::Events(vec!["quote.accepted".to_string()]),
            ),
            schedule_trigger("t3", daily_kind("09:00", None)),
            schedule_trigger("t4", daily_kind("09:00", Some("America/New_York"))),
        ];

        let seeds = schedule_seeds(&triggers, paris(), at("2026-11-06T07:00:00Z"));

        assert_eq!(
            seeds,
            vec![
                ScheduleSeed {
                    trigger_id: "t3".to_string(),
                    next_run_at: at("2026-11-06T08:00:00Z"),
                },
                ScheduleSeed {
                    trigger_id: "t4".to_string(),
                    next_run_at: at("2026-11-06T14:00:00Z"),
                },
            ]
        );
    }

    #[test]
    fn a_schedule_that_cannot_be_read_yields_no_seed() {
        let triggers = vec![
            schedule_trigger("t1", daily_kind("nope", None)),
            schedule_trigger("t2", daily_kind("09:00", Some("Mars/Olympus"))),
            schedule_trigger(
                "t3",
                TriggerKind::Schedule {
                    frequency: Frequency::Weekly,
                    time: "09:00".to_string(),
                    weekday: None,
                    timezone: None,
                },
            ),
        ];

        assert!(schedule_seeds(&triggers, paris(), at("2026-11-06T07:00:00Z")).is_empty());
    }

    fn due(kind: TriggerKind, organization_timezone: &str) -> DueSchedule {
        DueSchedule {
            org_id: OrganizationId(Uuid::from_u128(1)),
            workflow_id: Uuid::from_u128(2),
            workflow_version_id: Uuid::from_u128(3),
            trigger_id: "t1".to_string(),
            scheduled_for: at("2026-11-06T08:00:00Z"),
            trigger: kind,
            organization_timezone: organization_timezone.to_string(),
        }
    }

    #[test]
    fn the_occurrence_after_a_firing_reads_the_organization_zone_unless_the_trigger_has_one() {
        let now = at("2026-11-06T08:00:00Z");

        assert_eq!(
            due(daily_kind("09:00", None), "Europe/Paris").next_after(now),
            Some(at("2026-11-07T08:00:00Z"))
        );
        assert_eq!(
            due(
                daily_kind("09:00", Some("America/New_York")),
                "Europe/Paris"
            )
            .next_after(now),
            Some(at("2026-11-06T14:00:00Z"))
        );
    }

    #[test]
    fn a_schedule_with_no_usable_zone_has_no_next_occurrence() {
        let now = at("2026-11-06T08:00:00Z");

        assert_eq!(
            due(daily_kind("09:00", None), "Mars/Olympus").next_after(now),
            None
        );
        assert_eq!(
            due(daily_kind("09:00", Some("Mars/Olympus")), "Europe/Paris").next_after(now),
            None
        );
        assert_eq!(
            due(TriggerKind::Manual, "Europe/Paris").next_after(now),
            None
        );
    }

    #[test]
    fn a_missed_occurrence_is_followed_by_the_next_future_one_not_by_the_backlog() {
        let now = at("2026-11-10T12:00:00Z");

        let next = due(daily_kind("09:00", None), "Europe/Paris").next_after(now);

        assert_eq!(next, Some(at("2026-11-11T08:00:00Z")));
    }
}
