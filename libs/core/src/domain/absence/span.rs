use std::str::FromStr;

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::error::{AbsenceError, AbsenceOutcome};
use crate::Tz;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayPart {
    Morning,
    Afternoon,
}

impl DayPart {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Morning => "MORNING",
            Self::Afternoon => "AFTERNOON",
        }
    }
}

impl FromStr for DayPart {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "MORNING" => Ok(Self::Morning),
            "AFTERNOON" => Ok(Self::Afternoon),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsenceSpan {
    start_date: NaiveDate,
    start_part: DayPart,
    end_date: NaiveDate,
    end_part: DayPart,
}

fn instant(date: NaiveDate, time: NaiveTime, timezone: Tz) -> Result<DateTime<Utc>, CoreError> {
    timezone
        .from_local_datetime(&date.and_time(time))
        .earliest()
        .map(|local| local.with_timezone(&Utc))
        .ok_or_else(|| CoreError::Internal(format!("no valid local time on {date}")))
}

impl AbsenceSpan {
    pub fn new(
        start_date: NaiveDate,
        start_part: DayPart,
        end_date: NaiveDate,
        end_part: DayPart,
    ) -> AbsenceOutcome<Self> {
        let reversed = end_date < start_date
            || (end_date == start_date
                && start_part == DayPart::Afternoon
                && end_part == DayPart::Morning);
        if reversed {
            Err(AbsenceError::EndBeforeStart)
        } else {
            Ok(Self {
                start_date,
                start_part,
                end_date,
                end_part,
            })
        }
    }

    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    pub fn start_part(&self) -> DayPart {
        self.start_part
    }

    pub fn end_date(&self) -> NaiveDate {
        self.end_date
    }

    pub fn end_part(&self) -> DayPart {
        self.end_part
    }

    pub fn is_full_days(&self) -> bool {
        self.start_part == DayPart::Morning && self.end_part == DayPart::Afternoon
    }

    pub fn window(&self, timezone: Tz) -> Result<(DateTime<Utc>, DateTime<Utc>), CoreError> {
        let midnight = NaiveTime::MIN;
        let noon = NaiveTime::from_hms_opt(12, 0, 0).unwrap_or(midnight);
        let starts_at = match self.start_part {
            DayPart::Morning => instant(self.start_date, midnight, timezone)?,
            DayPart::Afternoon => instant(self.start_date, noon, timezone)?,
        };
        let ends_at = match self.end_part {
            DayPart::Morning => instant(self.end_date, noon, timezone)?,
            DayPart::Afternoon => {
                let next = self.end_date.succ_opt().ok_or_else(|| {
                    CoreError::Internal("the day after has a midnight".to_owned())
                })?;
                instant(next, midnight, timezone)?
            }
        };
        Ok((starts_at, ends_at))
    }

    pub fn from_window(
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        timezone: Tz,
    ) -> AbsenceOutcome<Self> {
        if ends_at <= starts_at {
            return Err(AbsenceError::EndBeforeStart);
        }
        let start = starts_at.with_timezone(&timezone);
        let end = ends_at.with_timezone(&timezone);

        let start_part = if start.hour() >= 12 {
            DayPart::Afternoon
        } else {
            DayPart::Morning
        };
        let ends_at_midnight = end.time() == NaiveTime::MIN;
        let (end_date, end_part) = if ends_at_midnight {
            (end.date_naive() - Duration::days(1), DayPart::Afternoon)
        } else if end.hour() < 12 || (end.hour() == 12 && end.minute() == 0 && end.second() == 0) {
            (end.date_naive(), DayPart::Morning)
        } else {
            (end.date_naive(), DayPart::Afternoon)
        };

        Self::new(start.date_naive(), start_part, end_date, end_part)
    }
}
