use chrono::{DateTime, Utc};
use common::CoreError;

use super::{DealAmount, DealStage, ports::DealRepository, service::convert_prospect};
use crate::{
    CustomerId, Quote, QuoteStatus, domain::customer::ports::CustomerRepository,
    domain::quote::service::QuoteChange,
};

pub struct DealQuoteLinks<D, C>
where
    D: DealRepository,
    C: CustomerRepository,
{
    deals: D,
    customers: C,
}

impl<D, C> DealQuoteLinks<D, C>
where
    D: DealRepository,
    C: CustomerRepository,
{
    pub fn new(deals: D, customers: C) -> Self {
        Self { deals, customers }
    }

    pub async fn follow(
        &mut self,
        change: &QuoteChange,
        now: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let quote = &change.quote;
        match (change.from, quote.status) {
            (from, QuoteStatus::Sent) if from != QuoteStatus::Sent => {
                self.quote_sent(quote, now).await
            }
            (from, QuoteStatus::Accepted) if from != QuoteStatus::Accepted => {
                self.quote_accepted(quote, now).await
            }
            _ => Ok(()),
        }
    }

    async fn quote_sent(&mut self, quote: &Quote, now: DateTime<Utc>) -> Result<(), CoreError> {
        let Some(deal_id) = quote.deal_id else {
            return Ok(());
        };
        let Some(mut deal) = self.deals.find_by_id(deal_id).await? else {
            return Ok(());
        };
        let Ok(amount) = DealAmount::try_from(i64::from(quote.net_cents)) else {
            return Ok(());
        };

        if matches!(deal.stage, DealStage::New | DealStage::Visit) {
            deal.move_to(DealStage::QuoteSent, now);
        }
        deal.amount = amount;
        self.deals.update(&deal).await?;

        Ok(())
    }

    async fn quote_accepted(&mut self, quote: &Quote, now: DateTime<Utc>) -> Result<(), CoreError> {
        let deal = match quote.deal_id {
            Some(deal_id) => self.deals.find_by_id(deal_id).await?,
            None => None,
        };

        match deal {
            Some(deal) if matches!(deal.stage, DealStage::Won) => Ok(()),
            Some(mut deal) => {
                let customer_id: CustomerId = deal.customer_id;
                deal.move_to(DealStage::Won, now);
                self.deals.update(&deal).await?;
                convert_prospect(&mut self.customers, customer_id, now).await
            }
            None => convert_prospect(&mut self.customers, quote.customer_id, now).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;
    use crate::{
        Customer, CustomerProfile, CustomerStatus, Deal, DealStage, OrganizationId, Quote, QuoteId,
        domain::{
            customer::ports::MockCustomerRepository, deal::fixtures::deal,
            deal::ports::MockDealRepository,
        },
    };

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap()
    }

    fn prospect(id: CustomerId) -> Customer {
        Customer {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            status: CustomerStatus::Prospect,
            name: "Duval".to_owned(),
            registration_number: None,
            phone: None,
            email: None,
            profile: CustomerProfile::default(),
            converted_at: None,
            archived_at: None,
            deleted_at: None,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn quote_of(deal: Option<&Deal>, customer_id: CustomerId, status: QuoteStatus) -> Quote {
        Quote {
            deal_id: deal.map(|deal| deal.id),
            status,
            net_cents: 87_500,
            gross_cents: 105_000,
            ..Quote::draft(
                QuoteId(Uuid::new_v4()),
                OrganizationId(Uuid::new_v4()),
                customer_id,
                crate::CustomerContextId(Uuid::new_v4()),
                "Cuisine".to_owned(),
                now(),
            )
        }
    }

    struct Fixture {
        deals: Arc<Mutex<Vec<Deal>>>,
        customers: Arc<Mutex<Vec<Customer>>>,
    }

    impl Fixture {
        fn new(deals: Vec<Deal>, customers: Vec<Customer>) -> Self {
            Self {
                deals: Arc::new(Mutex::new(deals)),
                customers: Arc::new(Mutex::new(customers)),
            }
        }

        fn links(&self) -> DealQuoteLinks<MockDealRepository, MockCustomerRepository> {
            let mut deals = MockDealRepository::new();
            let store = self.deals.clone();
            deals.expect_find_by_id().returning(move |id| {
                let found = store.lock().unwrap().iter().find(|d| d.id == id).cloned();
                Box::pin(async move { Ok(found) })
            });
            let store = self.deals.clone();
            deals.expect_update().returning(move |deal| {
                let mut guard = store.lock().unwrap();
                let slot = guard.iter_mut().find(|d| d.id == deal.id).unwrap();
                *slot = deal.clone();
                let saved = deal.clone();
                Box::pin(async move { Ok(saved) })
            });

            let mut customers = MockCustomerRepository::new();
            let store = self.customers.clone();
            customers.expect_find_by_id().returning(move |id| {
                let found = store.lock().unwrap().iter().find(|c| c.id == id).cloned();
                Box::pin(async move { Ok(found) })
            });
            let store = self.customers.clone();
            customers.expect_update().returning(move |customer| {
                let mut guard = store.lock().unwrap();
                let slot = guard.iter_mut().find(|c| c.id == customer.id).unwrap();
                *slot = customer.clone();
                let saved = customer.clone();
                Box::pin(async move { Ok(saved) })
            });

            DealQuoteLinks::new(deals, customers)
        }

        fn deal(&self, id: crate::DealId) -> Deal {
            self.deals
                .lock()
                .unwrap()
                .iter()
                .find(|deal| deal.id == id)
                .cloned()
                .unwrap()
        }

        fn customer(&self, id: CustomerId) -> Customer {
            self.customers
                .lock()
                .unwrap()
                .iter()
                .find(|customer| customer.id == id)
                .cloned()
                .unwrap()
        }
    }

    fn change(quote: Quote, from: QuoteStatus) -> QuoteChange {
        QuoteChange { quote, from }
    }

    #[tokio::test]
    async fn spec_crm_4_4_sending_moves_a_new_or_visit_deal_to_quote_sent_with_the_subtotal() {
        for stage in [DealStage::New, DealStage::Visit] {
            let linked = deal(stage);
            let customer_id = linked.customer_id;
            let fixture = Fixture::new(vec![linked.clone()], vec![prospect(customer_id)]);
            let quote = quote_of(Some(&linked), customer_id, QuoteStatus::Sent);

            fixture
                .links()
                .follow(&change(quote, QuoteStatus::Draft), now())
                .await
                .unwrap();

            let moved = fixture.deal(linked.id);
            assert_eq!(moved.stage, DealStage::QuoteSent);
            assert_eq!(moved.amount.get(), 87_500);
            assert_eq!(moved.stage_changed_at, now());
        }
    }

    #[tokio::test]
    async fn sending_keeps_the_stage_of_a_deal_that_is_past_the_visit_but_updates_its_amount() {
        let linked = deal(DealStage::Negotiation);
        let customer_id = linked.customer_id;
        let fixture = Fixture::new(vec![linked.clone()], vec![prospect(customer_id)]);
        let quote = quote_of(Some(&linked), customer_id, QuoteStatus::Sent);

        fixture
            .links()
            .follow(&change(quote, QuoteStatus::Draft), now())
            .await
            .unwrap();

        let kept = fixture.deal(linked.id);
        assert_eq!(kept.stage, DealStage::Negotiation);
        assert_eq!(kept.amount.get(), 87_500);
        assert_eq!(kept.stage_changed_at, linked.stage_changed_at);
    }

    #[tokio::test]
    async fn a_quote_without_a_deal_leaves_the_pipeline_alone_when_sent() {
        let linked = deal(DealStage::New);
        let customer_id = linked.customer_id;
        let fixture = Fixture::new(vec![linked.clone()], vec![prospect(customer_id)]);
        let quote = quote_of(None, customer_id, QuoteStatus::Sent);

        fixture
            .links()
            .follow(&change(quote, QuoteStatus::Draft), now())
            .await
            .unwrap();

        assert_eq!(fixture.deal(linked.id), linked);
    }

    #[tokio::test]
    async fn spec_crm_4_6_accepting_wins_the_deal_and_converts_the_prospect() {
        let linked = deal(DealStage::QuoteSent);
        let customer_id = linked.customer_id;
        let fixture = Fixture::new(vec![linked.clone()], vec![prospect(customer_id)]);
        let quote = quote_of(Some(&linked), customer_id, QuoteStatus::Accepted);

        fixture
            .links()
            .follow(&change(quote, QuoteStatus::Sent), now())
            .await
            .unwrap();

        let won = fixture.deal(linked.id);
        assert_eq!(won.stage, DealStage::Won);
        assert_eq!(won.closed_at, Some(now()));
        let converted = fixture.customer(customer_id);
        assert_eq!(converted.status, CustomerStatus::Client);
        assert_eq!(converted.converted_at, Some(now()));
    }

    #[tokio::test]
    async fn spec_crm_4_6_accepting_a_quote_without_a_deal_converts_the_prospect() {
        let customer_id = CustomerId(Uuid::new_v4());
        let fixture = Fixture::new(Vec::new(), vec![prospect(customer_id)]);
        let quote = quote_of(None, customer_id, QuoteStatus::Accepted);

        fixture
            .links()
            .follow(&change(quote, QuoteStatus::Sent), now())
            .await
            .unwrap();

        assert_eq!(fixture.customer(customer_id).status, CustomerStatus::Client);
    }

    #[tokio::test]
    async fn accepting_leaves_an_already_won_deal_and_a_client_untouched() {
        let linked = deal(DealStage::Won);
        let customer_id = linked.customer_id;
        let client = Customer {
            status: CustomerStatus::Client,
            converted_at: Some(now()),
            ..prospect(customer_id)
        };
        let fixture = Fixture::new(vec![linked.clone()], vec![client.clone()]);
        let quote = quote_of(Some(&linked), customer_id, QuoteStatus::Accepted);

        fixture
            .links()
            .follow(&change(quote, QuoteStatus::Sent), now())
            .await
            .unwrap();

        assert_eq!(fixture.deal(linked.id), linked);
        assert_eq!(fixture.customer(customer_id), client);
    }

    #[tokio::test]
    async fn declining_and_restating_touch_nothing() {
        let linked = deal(DealStage::QuoteSent);
        let customer_id = linked.customer_id;
        let fixture = Fixture::new(vec![linked.clone()], vec![prospect(customer_id)]);

        fixture
            .links()
            .follow(
                &change(
                    quote_of(Some(&linked), customer_id, QuoteStatus::Declined),
                    QuoteStatus::Sent,
                ),
                now(),
            )
            .await
            .unwrap();
        fixture
            .links()
            .follow(
                &change(
                    quote_of(Some(&linked), customer_id, QuoteStatus::Sent),
                    QuoteStatus::Sent,
                ),
                now(),
            )
            .await
            .unwrap();

        assert_eq!(fixture.deal(linked.id), linked);
        assert_eq!(
            fixture.customer(customer_id).status,
            CustomerStatus::Prospect
        );
    }
}
