use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::Utc;
use mestier_core::{
    BillingSettings, InvoiceNumberSeries, MockInvoiceRepository, MockMemberRepository,
    MockOrganizationRepository, MockQuoteRepository, MockRoleRepository, NumberSeries,
    Organization, UserId,
};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct State {
    pub settings: Option<BillingSettings>,
    pub counters: HashMap<(NumberSeries, i32), i32>,
    pub writes: usize,
}

pub type Store = Arc<Mutex<State>>;

pub fn state(store: &Store) -> MutexGuard<'_, State> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn organization(id: mestier_core::OrganizationId) -> Organization {
    let now = Utc::now();
    Organization {
        id,
        name: "Scenario company".into(),
        slug: "scenario-company".into(),
        owner_id: UserId(Uuid::new_v4()),
        legal_name: None,
        legal_form: None,
        registration_number: None,
        vat_status: None,
        share_capital_cents: None,
        address_line1: None,
        address_line2: None,
        address_postal_code: None,
        address_city: None,
        address_country: None,
        contact_email: None,
        contact_phone: None,
        insurance_mention: None,
        quote_number_prefix: "DEV".to_owned(),
        invoice_number_prefix: "FAC".to_owned(),
        field_clock_enabled: false,
        vat_on_debits: false,
        mcp_enabled: false,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

pub fn organizations(store: &Store) -> MockOrganizationRepository {
    let mut repo = MockOrganizationRepository::new();

    repo.expect_find_by_id().returning(move |id| {
        let found = organization(id);
        Box::pin(async move { Ok(Some(found)) })
    });

    let store_ref = store.clone();
    repo.expect_find_billing_settings().returning(move |_| {
        let settings = state(&store_ref).settings.clone();
        Box::pin(async move { Ok(settings) })
    });

    let store_ref = store.clone();
    repo.expect_upsert_billing_settings()
        .returning(move |_, settings| {
            let mut guard = state(&store_ref);
            guard.settings = Some(settings.clone());
            guard.writes += 1;
            let stored = settings.clone();
            Box::pin(async move { Ok(stored) })
        });

    repo
}

pub fn quotes(store: &Store) -> MockQuoteRepository {
    let mut repo = MockQuoteRepository::new();

    let store_ref = store.clone();
    repo.expect_last_assigned_number()
        .returning(move |_, year| {
            let last = counter(&store_ref, NumberSeries::Quote, year);
            Box::pin(async move { Ok(last) })
        });

    let store_ref = store.clone();
    repo.expect_raise_last_assigned_number()
        .returning(move |_, year, last| {
            raise(&store_ref, NumberSeries::Quote, year, last);
            Box::pin(async move { Ok(()) })
        });

    repo
}

fn series_of(series: InvoiceNumberSeries) -> NumberSeries {
    match series {
        InvoiceNumberSeries::Invoice => NumberSeries::Invoice,
        InvoiceNumberSeries::CreditNote => NumberSeries::CreditNote,
    }
}

pub fn invoices(store: &Store) -> MockInvoiceRepository {
    let mut repo = MockInvoiceRepository::new();

    let store_ref = store.clone();
    repo.expect_last_assigned_number()
        .returning(move |_, series, year| {
            let last = counter(&store_ref, series_of(series), year);
            Box::pin(async move { Ok(last) })
        });

    let store_ref = store.clone();
    repo.expect_raise_last_assigned_number()
        .returning(move |_, series, year, last| {
            raise(&store_ref, series_of(series), year, last);
            Box::pin(async move { Ok(()) })
        });

    repo
}

pub fn members() -> MockMemberRepository {
    MockMemberRepository::new()
}

pub fn roles() -> MockRoleRepository {
    MockRoleRepository::new()
}

pub fn counter(store: &Store, series: NumberSeries, year: i32) -> i32 {
    state(store)
        .counters
        .get(&(series, year))
        .copied()
        .unwrap_or(0)
}

fn raise(store: &Store, series: NumberSeries, year: i32, last: i32) {
    let mut guard = state(store);
    let entry = guard.counters.entry((series, year)).or_insert(0);
    *entry = (*entry).max(last);
}
