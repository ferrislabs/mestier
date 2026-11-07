use chrono::{DateTime, TimeZone, Utc};
use common::CoreError;
use events::testing::RecordingEmitter;
use mestier_core::{
    CustomerContextId, CustomerId, MockQuoteRepository, OrganizationId, Quote, QuoteError, QuoteId,
    QuoteService,
};

use crate::repository::{self, Client, Company, Doubles, Store};

pub type Service<'a> = QuoteService<MockQuoteRepository, &'a RecordingEmitter>;

#[derive(Debug, Default, cucumber::World)]
pub struct QuoteWorld {
    pub store: Store,
    pub emitter: RecordingEmitter,
    pub organization_id: Option<OrganizationId>,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    pub source_id: Option<QuoteId>,
    pub outcome: Option<Result<Quote, CoreError>>,
    pub refusal: Option<QuoteError>,
    pub today: Option<DateTime<Utc>>,
    pub subject_to_vat: bool,
    pub without_billing_address: bool,
    pub validity_days: Option<i64>,
}

impl QuoteWorld {
    pub fn service(&self) -> Service<'_> {
        QuoteService::new(repository::stubbed(&self.store), &self.emitter)
    }

    pub fn doubles(&self) -> Doubles {
        repository::doubles(
            &Company {
                id: self.organization_id(),
                subject_to_vat: self.subject_to_vat,
                validity_days: self.validity_days.unwrap_or(30),
            },
            &Client {
                id: self.customer_id(),
                address_id: self.customer_context_id(),
                has_billing_address: !self.without_billing_address,
            },
        )
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.today
            .unwrap_or_else(|| Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap())
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

    pub fn quote(&self, id: QuoteId) -> Quote {
        repository::quotes(&self.store)
            .iter()
            .find(|quote| quote.id == id)
            .cloned()
            .expect("the quote the scenario acted on is in the store")
    }

    pub fn current_quote(&self) -> Quote {
        self.quote(
            self.quote_id
                .expect("a scenario creates or seeds a quote before asserting on it"),
        )
    }

    pub fn record(&mut self, outcome: Result<Quote, CoreError>) {
        if let Ok(quote) = &outcome {
            self.quote_id = Some(quote.id);
        }
        self.refusal = None;
        self.outcome = Some(outcome);
    }

    pub fn record_outcome(&mut self, outcome: Result<Result<Quote, QuoteError>, CoreError>) {
        match outcome {
            Ok(Ok(quote)) => self.record(Ok(quote)),
            Ok(Err(refusal)) => {
                self.outcome = None;
                self.refusal = Some(refusal);
            }
            Err(error) => self.record(Err(error)),
        }
    }
}
