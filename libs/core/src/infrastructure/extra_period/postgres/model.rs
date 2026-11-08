use chrono::{DateTime, NaiveDate, NaiveTime, Timelike, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    ExtraPeriod, ExtraPeriodId, ExtraReason, ExtraStatus, LunchInput, MemberId, OrganizationId,
    RateBp, SurchargeId, WorkDay, WorkDayInput,
};

#[derive(Debug, Clone)]
pub struct ExtraPeriodRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub lunch_start: Option<NaiveTime>,
    pub lunch_end: Option<NaiveTime>,
    pub reason: String,
    pub surcharge_id: Option<Uuid>,
    pub rate_bp: i32,
    pub status: String,
    pub created_by_member_id: Option<Uuid>,
    pub decided_by_member_id: Option<Uuid>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, what: &str) -> CoreError {
    CoreError::Internal(format!("extra_periods {id}: invalid {what}"))
}

fn clock(time: NaiveTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

#[derive(Debug, Clone, Copy)]
pub struct HoursColumns {
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub lunch_start: Option<NaiveTime>,
    pub lunch_end: Option<NaiveTime>,
}

impl From<&WorkDay> for HoursColumns {
    fn from(day: &WorkDay) -> Self {
        use crate::infrastructure::day_override::postgres::model::to_time;

        Self {
            start_time: to_time(day.start()),
            end_time: to_time(day.end()),
            lunch_start: day.lunch().map(|lunch| to_time(lunch.start())),
            lunch_end: day.lunch().map(|lunch| to_time(lunch.end())),
        }
    }
}

impl TryFrom<ExtraPeriodRow> for ExtraPeriod {
    type Error = CoreError;

    fn try_from(row: ExtraPeriodRow) -> Result<Self, Self::Error> {
        let lunch = match (row.lunch_start, row.lunch_end) {
            (Some(from), Some(to)) => Some(LunchInput {
                start: clock(from),
                end: clock(to),
            }),
            _ => None,
        };
        let hours = WorkDay::try_from(&WorkDayInput {
            start: clock(row.start_time),
            end: clock(row.end_time),
            lunch,
        })
        .map_err(|_| corrupt(row.id, "hours"))?;
        Ok(Self {
            id: ExtraPeriodId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            date: row.date,
            hours,
            reason: ExtraReason::try_from(row.reason.as_str())
                .map_err(|_| corrupt(row.id, "reason"))?,
            surcharge_id: row.surcharge_id.map(SurchargeId),
            rate: RateBp::try_from(row.rate_bp).map_err(|_| corrupt(row.id, "rate_bp"))?,
            status: row
                .status
                .parse::<ExtraStatus>()
                .map_err(|()| corrupt(row.id, "status"))?,
            created_by_member_id: row.created_by_member_id.map(MemberId),
            decided_by_member_id: row.decided_by_member_id.map(MemberId),
            decided_at: row.decided_at,
            decision_comment: row.decision_comment,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
