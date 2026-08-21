use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, domain::profitability::ProfitabilityFacts};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ProfitabilityRepository: Send {
    /// Every fact the calculation needs, for one organization over one window.
    ///
    /// One call rather than a method per list: the four sets are read together
    /// or not at all, and an adapter that split them would open the door to a
    /// per-project query.
    ///
    /// Bounds are instants rather than dates, because a calendar day only means
    /// something once a timezone is chosen and that is the use case's job, not
    /// the adapter's. Half-open, `from` included and `to` excluded.
    ///
    /// A project is in scope when at least one of its planned tasks overlaps the
    /// window. Filtering is on when the work is *planned*, not on when the
    /// project was created: a project opened in June and worked in July belongs
    /// to July. Cancelled tasks are excluded — a task that will not happen costs
    /// nothing.
    fn load(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> impl Future<Output = Result<ProfitabilityFacts, CoreError>> + Send;
}
