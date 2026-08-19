pub mod profitability;
pub mod worked_hours;

use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::IntoParams;

/// The period a report covers, both ends included.
///
/// Days rather than instants: a period is something a person picks in a date
/// picker, and turning it into instants needs the organization's timezone,
/// which the use case owns.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PeriodQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}
