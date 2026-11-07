use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::{DateTime, Utc};
use mestier_core::{
    Customer, CustomerId, Deal, DealId, DealStage, DealStageName, Member, MemberId,
    MockCustomerRepository, MockDealRepository, MockMemberRepository, MockRoleRepository,
    StageTotal,
};

#[derive(Debug, Default)]
pub struct State {
    pub customers: HashMap<CustomerId, Customer>,
    pub deals: HashMap<DealId, Deal>,
    pub members: HashMap<MemberId, Member>,
}

pub type Store = Arc<Mutex<State>>;

pub fn state(store: &Store) -> MutexGuard<'_, State> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn grouped(deals: Vec<Deal>) -> Vec<StageTotal> {
    DealStageName::ALL
        .into_iter()
        .filter_map(|stage| {
            let amounts: Vec<i64> = deals
                .iter()
                .filter(|deal| deal.stage.name() == stage)
                .map(|deal| deal.amount.get())
                .collect();
            (!amounts.is_empty()).then(|| StageTotal {
                stage,
                count: amounts.len() as i64,
                amount_cents: amounts.iter().sum(),
            })
        })
        .collect()
}

pub fn deals(store: &Store) -> MockDealRepository {
    let mut repo = MockDealRepository::new();

    let store_ref = store.clone();
    repo.expect_insert().returning(move |deal| {
        let stored = deal.clone();
        state(&store_ref).deals.insert(stored.id, stored.clone());
        Box::pin(async move { Ok(stored) })
    });

    let store_ref = store.clone();
    repo.expect_update().returning(move |deal| {
        let stored = deal.clone();
        state(&store_ref).deals.insert(stored.id, stored.clone());
        Box::pin(async move { Ok(stored) })
    });

    let store_ref = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = state(&store_ref).deals.get(&id).cloned();
        Box::pin(async move { Ok(found) })
    });

    let store_ref = store.clone();
    repo.expect_delete().returning(move |id| {
        state(&store_ref).deals.remove(&id);
        Box::pin(async { Ok(()) })
    });

    let store_ref = store.clone();
    repo.expect_lose_open_of_customer()
        .returning(move |customer_id, reason, now| {
            let mut guard = state(&store_ref);
            let mut changed = 0;
            for deal in guard
                .deals
                .values_mut()
                .filter(|deal| deal.customer_id == customer_id && deal.stage.is_open())
            {
                let lost = DealStage::Lost(reason.clone());
                deal.move_to(lost, now);
                changed += 1;
            }
            Box::pin(async move { Ok(changed) })
        });

    let store_ref = store.clone();
    repo.expect_stage_totals()
        .returning(move |organization_id| {
            let all: Vec<Deal> = state(&store_ref)
                .deals
                .values()
                .filter(|deal| deal.organization_id == organization_id)
                .cloned()
                .collect();
            Box::pin(async move { Ok(grouped(all)) })
        });

    let store_ref = store.clone();
    repo.expect_closed_totals_since()
        .returning(move |organization_id, since: DateTime<Utc>| {
            let closed: Vec<Deal> = state(&store_ref)
                .deals
                .values()
                .filter(|deal| {
                    deal.organization_id == organization_id
                        && deal.closed_at.is_some_and(|closed_at| closed_at >= since)
                })
                .cloned()
                .collect();
            Box::pin(async move { Ok(grouped(closed)) })
        });

    repo
}

pub fn customers(store: &Store) -> MockCustomerRepository {
    let mut repo = MockCustomerRepository::new();

    let store_ref = store.clone();
    repo.expect_insert().returning(move |customer| {
        let stored = customer.clone();
        state(&store_ref)
            .customers
            .insert(stored.id, stored.clone());
        Box::pin(async move { Ok(stored) })
    });

    let store_ref = store.clone();
    repo.expect_update().returning(move |customer| {
        let stored = customer.clone();
        state(&store_ref)
            .customers
            .insert(stored.id, stored.clone());
        Box::pin(async move { Ok(stored) })
    });

    let store_ref = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = state(&store_ref).customers.get(&id).cloned();
        Box::pin(async move { Ok(found) })
    });

    repo.expect_has_contact()
        .returning(|_, _| Box::pin(async { Ok(false) }));
    repo.expect_has_address()
        .returning(|_, _| Box::pin(async { Ok(false) }));

    let store_ref = store.clone();
    repo.expect_count_converted_since()
        .returning(move |organization_id, since| {
            let count = state(&store_ref)
                .customers
                .values()
                .filter(|customer| {
                    customer.organization_id == organization_id
                        && customer
                            .converted_at
                            .is_some_and(|converted_at| converted_at >= since)
                })
                .count() as i64;
            Box::pin(async move { Ok(count) })
        });

    repo
}

pub fn members(store: &Store) -> MockMemberRepository {
    let mut repo = MockMemberRepository::new();

    let store_ref = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = state(&store_ref).members.get(&id).cloned();
        Box::pin(async move { Ok(found) })
    });

    repo
}

pub fn roles() -> MockRoleRepository {
    MockRoleRepository::new()
}
