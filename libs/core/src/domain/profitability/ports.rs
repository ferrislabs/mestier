use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, domain::profitability::ProfitabilityFacts};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ProfitabilityRepository: Send {
    /// Every fact the calculation needs, for one organization over one window.
    ///
    /// One call rather than a method per list: the three sets are read together
    /// or not at all, and an adapter that split them would open the door to a
    /// per-job query.
    ///
    /// Bounds are instants rather than dates, because a calendar day only means
    /// something once a timezone is chosen and that is the use case's job, not
    /// the adapter's. Half-open, `from` included and `to` excluded, so
    /// consecutive periods neither overlap nor leave a gap.
    ///
    /// Filtering is on when the work *happened*, not on when the job was
    /// planned: a chantier planned in June and worked in July belongs to July.
    fn load(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> impl Future<Output = Result<ProfitabilityFacts, CoreError>> + Send;
}
