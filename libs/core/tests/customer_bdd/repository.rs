use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use mestier_core::{
    Customer, CustomerContact, CustomerContext, CustomerId, Member, MemberId,
    MockCustomerContactRepository, MockCustomerContextRepository, MockCustomerRepository,
    MockDealRepository, MockMemberRepository, MockRoleRepository,
};

#[derive(Debug, Default)]
pub struct State {
    pub customers: HashMap<CustomerId, Customer>,
    pub contacts: Vec<CustomerContact>,
    pub contexts: Vec<CustomerContext>,
    pub members: HashMap<MemberId, Member>,
}

pub type Store = Arc<Mutex<State>>;

pub fn state(store: &Store) -> MutexGuard<'_, State> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

    let store_ref = store.clone();
    repo.expect_has_contact()
        .returning(move |customer_id, contact_id| {
            let owned = state(&store_ref)
                .contacts
                .iter()
                .any(|contact| contact.id == contact_id && contact.customer_id == customer_id);
            Box::pin(async move { Ok(owned) })
        });

    let store_ref = store.clone();
    repo.expect_has_address()
        .returning(move |customer_id, address_id| {
            let owned = state(&store_ref)
                .contexts
                .iter()
                .any(|context| context.id == address_id && context.customer_id == customer_id);
            Box::pin(async move { Ok(owned) })
        });

    repo
}

pub fn contacts(store: &Store) -> MockCustomerContactRepository {
    let mut repo = MockCustomerContactRepository::new();

    let store_ref = store.clone();
    repo.expect_insert().returning(move |contact| {
        let stored = contact.clone();
        state(&store_ref).contacts.push(stored.clone());
        Box::pin(async move { Ok(stored) })
    });

    repo
}

pub fn contexts(store: &Store) -> MockCustomerContextRepository {
    let mut repo = MockCustomerContextRepository::new();

    let store_ref = store.clone();
    repo.expect_insert().returning(move |context| {
        let stored = context.clone();
        state(&store_ref).contexts.push(stored.clone());
        Box::pin(async move { Ok(stored) })
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

pub fn deals() -> MockDealRepository {
    let mut repo = MockDealRepository::new();
    repo.expect_list()
        .returning(|_, _, _| Box::pin(async { Ok((Vec::new(), 0)) }));
    repo.expect_lose_open_of_customer()
        .returning(|_, _, _| Box::pin(async { Ok(0) }));
    repo
}
