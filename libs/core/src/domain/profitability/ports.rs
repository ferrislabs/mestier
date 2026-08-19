use common::CoreError;

use crate::{DateRange, OrganizationId, domain::profitability::ProfitabilityFacts};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ProfitabilityRepository: Send {
    /// Every fact the calculation needs, for one organization over one period.
    ///
    /// One call rather than a method per list: the three sets are read together
    /// or not at all, and an adapter that splits them would open the door to a
    /// per-job query.
    ///
    /// Filtering is on when the work *happened*, not on when the job was
    /// planned: a chantier planned in June and worked in July belongs to July's
    /// costs.
    fn load(
        &mut self,
        organization_id: OrganizationId,
        range: DateRange,
    ) -> impl Future<Output = Result<ProfitabilityFacts, CoreError>> + Send;
}
