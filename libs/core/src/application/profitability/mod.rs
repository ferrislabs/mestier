use chrono::{NaiveDate, TimeZone, Utc};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, ProfitabilityReport, Tz,
    application::{MestierUseCase, organization::resolve_timezone},
    domain::profitability::{ReportPeriod, service::ProfitabilityService},
};

mod tests;

impl MestierUseCase {
    /// Profitability for one organization over one calendar period.
    ///
    /// The dates are resolved into instants here, in the organization's
    /// timezone, for the same reason the field app does it: a period is a
    /// business notion and UTC is not where the business lives. The timezone
    /// then travels with the period, because an all-day task has no duration
    /// until one is chosen.
    ///
    /// `planning` is listed for its two organization-wide work-time queries,
    /// which only all-day tasks need — see `ProfitabilityService`.
    #[transactional(profitability, planning, organization)]
    pub async fn profitability_report(
        &self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<ProfitabilityReport, CoreError> {
        if to < from {
            return Err(CoreError::Conflict(
                "the end of the period cannot precede its start".to_owned(),
            ));
        }

        let mut organization_repository = organization_repository;
        let timezone = resolve_timezone(&mut organization_repository, organization_id).await?;
        let (starts_at, ends_at) = period_bounds(from, to, timezone)?;

        let mut service = ProfitabilityService::new(profitability_repository, planning_repository);

        service
            .report(
                organization_id,
                ReportPeriod {
                    from: starts_at,
                    to: ends_at,
                    timezone,
                },
            )
            .await
    }
}

/// The half-open instant window covering `from` through `to` inclusive.
///
/// `to` is a day the caller wants included, so the upper bound is the midnight
/// that opens the day after. Uses the earliest valid local midnight: on a
/// spring-forward night the wall clock skips an hour and asking for the exact
/// one finds nothing.
fn period_bounds(
    from: NaiveDate,
    to: NaiveDate,
    timezone: Tz,
) -> Result<(chrono::DateTime<Utc>, chrono::DateTime<Utc>), CoreError> {
    let local_midnight = |date: NaiveDate| -> Result<chrono::DateTime<Utc>, CoreError> {
        let naive = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| CoreError::Internal("midnight exists on every date".to_owned()))?;

        timezone
            .from_local_datetime(&naive)
            .earliest()
            .ok_or_else(|| CoreError::Internal("no valid local midnight".to_owned()))
            .map(|instant| instant.with_timezone(&Utc))
    };

    let day_after_to = to
        .succ_opt()
        .ok_or_else(|| CoreError::Internal("the day after the period exists".to_owned()))?;

    Ok((local_midnight(from)?, local_midnight(day_after_to)?))
}
