//! The state a scenario carries from one step to the next.
//!
//! It knows the domain service and the two ports it depends on — a repository
//! and an event emitter — and nothing about Postgres, Axum or the transaction
//! machinery. That is the point: these scenarios describe business rules, so
//! they must keep failing for business reasons only.

use common::CoreError;
use events::testing::RecordingEmitter;
use mestier_core::{
    CustomerContextId, CustomerId, MockQuoteRepository, OrganizationId, Quote, QuoteId,
    QuoteService,
};

use crate::repository::{self, Store};

/// The service under test, wired to the doubles this suite owns.
///
/// The emitter is borrowed rather than moved so a `Then` step can still read
/// what was published after the service has done its work.
pub type Service<'a> = QuoteService<MockQuoteRepository, &'a RecordingEmitter>;

#[derive(Debug, Default, cucumber::World)]
pub struct QuoteWorld {
    pub store: Store,
    pub emitter: RecordingEmitter,
    /// Who the scenario is acting for. Every quote is scoped to one company.
    pub organization_id: Option<OrganizationId>,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    /// The quote the last `When` produced or the last `Given` seeded.
    pub quote_id: Option<QuoteId>,
    /// The outcome of the last write, kept whole so a scenario can assert on a
    /// refusal instead of unwinding on it.
    pub outcome: Option<Result<Quote, CoreError>>,
}

impl QuoteWorld {
    /// Builds a service over a mock freshly stubbed against the shared store.
    /// Short-lived by design: it holds a borrow of the emitter, so a step drops
    /// it before asserting.
    pub fn service(&self) -> Service<'_> {
        QuoteService::new(repository::stubbed(&self.store), &self.emitter)
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
            .expect("a scenario states the company in its `Given`")
    }

    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
            .expect("a scenario states the customer in its `Given`")
    }

    pub fn customer_context_id(&self) -> CustomerContextId {
        self.customer_context_id
            .expect("a scenario states the customer in its `Given`")
    }

    /// The quote the scenario is talking about, reloaded from the store so an
    /// assertion reads what was persisted rather than what was returned.
    pub fn current_quote(&self) -> Quote {
        let id = self
            .quote_id
            .expect("a scenario creates or seeds a quote before asserting on it");

        repository::quotes(&self.store)
            .iter()
            .find(|quote| quote.id == id)
            .cloned()
            .expect("the quote the scenario just acted on is in the store")
    }

    /// Records what a write returned, and remembers the quote it produced so
    /// the following steps can talk about "the quote" without naming it again.
    pub fn record(&mut self, outcome: Result<Quote, CoreError>) {
        if let Ok(quote) = &outcome {
            self.quote_id = Some(quote.id);
        }
        self.outcome = Some(outcome);
    }
}
