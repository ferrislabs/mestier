use chrono::{DateTime, NaiveDate, NaiveTime, Timelike, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    ClockTime, DayOverride, DayOverrideId, DayShape, LunchInput, MemberId, OrganizationId,
    OverrideKind, WorkDay, WorkDayInput,
};

#[derive(Debug, Clone)]
pub struct DayOverrideRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub date: NaiveDate,
    pub kind: String,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub lunch_start: Option<NaiveTime>,
    pub lunch_end: Option<NaiveTime>,
    pub note: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, what: &str) -> CoreError {
    CoreError::Internal(format!("day_overrides {id}: invalid {what}"))
}

pub fn to_time(time: ClockTime) -> NaiveTime {
    let minutes = u32::from(time.minutes());
    NaiveTime::from_hms_opt(minutes / 60, minutes % 60, 0).unwrap_or(NaiveTime::MIN)
}

fn clock(time: NaiveTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ShapeColumns {
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub lunch_start: Option<NaiveTime>,
    pub lunch_end: Option<NaiveTime>,
}

impl From<&DayShape> for ShapeColumns {
    fn from(shape: &DayShape) -> Self {
        match shape.work() {
            None => Self::default(),
            Some(day) => Self {
                start_time: Some(to_time(day.start())),
                end_time: Some(to_time(day.end())),
                lunch_start: day.lunch().map(|lunch| to_time(lunch.start())),
                lunch_end: day.lunch().map(|lunch| to_time(lunch.end())),
            },
        }
    }
}

impl TryFrom<DayOverrideRow> for DayOverride {
    type Error = CoreError;

    fn try_from(row: DayOverrideRow) -> Result<Self, Self::Error> {
        let kind = row
            .kind
            .parse::<OverrideKind>()
            .map_err(|()| corrupt(row.id, "kind"))?;
        let shape = match kind {
            OverrideKind::Off => DayShape::Off,
            OverrideKind::Work => {
                let (Some(start), Some(end)) = (row.start_time, row.end_time) else {
                    return Err(corrupt(row.id, "hours"));
                };
                let lunch = match (row.lunch_start, row.lunch_end) {
                    (Some(from), Some(to)) => Some(LunchInput {
                        start: clock(from),
                        end: clock(to),
                    }),
                    _ => None,
                };
                let input = WorkDayInput {
                    start: clock(start),
                    end: clock(end),
                    lunch,
                };
                DayShape::Work(WorkDay::try_from(&input).map_err(|_| corrupt(row.id, "hours"))?)
            }
        };
        Ok(Self {
            id: DayOverrideId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            date: row.date,
            shape,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
