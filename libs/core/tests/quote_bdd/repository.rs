//! The `QuoteRepository` double the scenarios run against.
//!
//! `mockall::automock` derives the type from the port itself, so there is no
//! hand-written adapter to keep in step with the trait. What it does not derive
//! is state: a mock is moved into the `QuoteService` and cannot be read back,
//! so the quotes live in a store the `World` owns and every stub closes over.
//!
//! Only the methods these scenarios exercise are stubbed. Reaching an unstubbed
//! one fails the step with mockall's own message, which names the method, and
//! adding it here is then four lines.

use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Utc;
use common::CoreError;
use mestier_core::{MockOrganizationRepository, MockQuoteRepository, Organization, Quote, UserId};
use uuid::Uuid;

/// Shared between the `World` and every mock built from it.
pub type Store = Arc<Mutex<Vec<Quote>>>;

/// The stored quotes. Locked for the length of a synchronous block and never
/// across an `await`, which is what keeps the stubbed futures `Send`. A
/// poisoned mutex means a step already panicked and the run is over.
pub fn quotes(store: &Store) -> MutexGuard<'_, Vec<Quote>> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A fresh mock over `store`, rebuilt on every step because the previous one
/// was consumed by the service it was handed to.
pub fn stubbed(store: &Store) -> MockQuoteRepository {
    let mut repo = MockQuoteRepository::new();

    // Same `{prefix}-{year}-NNNN` shape as the Postgres adapter, and derived
    // from what the store holds, so the scenario stating a clean year is
    // what produces `0001` rather than the stub asserting its own constant.
    let store_ref = store.clone();
    repo.expect_allocate_number()
        .returning(move |org, prefix, year| {
            let needle = format!("{prefix}-{year}-");
            let count = quotes(&store_ref)
                .iter()
                .filter(|quote| {
                    quote.organization_id == org
                        && quote
                            .reference
                            .as_ref()
                            .is_some_and(|reference| reference.starts_with(&needle))
                })
                .count();
            let reference = format!("{needle}{:04}", count + 1);

            Box::pin(async move { Ok(reference) })
        });

    let store_ref = store.clone();
    repo.expect_insert().returning(move |quote| {
        quotes(&store_ref).push(quote.clone());
        let quote = quote.clone();

        Box::pin(async move { Ok(quote) })
    });

    let store_ref = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = quotes(&store_ref)
            .iter()
            .find(|quote| quote.id == id)
            .cloned();

        Box::pin(async move { Ok(found) })
    });

    let store_ref = store.clone();
    repo.expect_update_status()
        .returning(move |id, status, reference, updated_at| {
            let outcome = match quotes(&store_ref).iter_mut().find(|quote| quote.id == id) {
                Some(quote) => {
                    quote.status = status;
                    if let Some(reference) = reference {
                        quote.reference = Some(reference);
                    }
                    quote.updated_at = updated_at;
                    Ok(quote.clone())
                }
                None => Err(CoreError::NotFound),
            };

            Box::pin(async move { outcome })
        });

    repo
}

/// A stub for the one thing `QuoteService` reads off the organization: its
/// VAT status. No scenario in `quote.feature` states one, so every company
/// here is unregistered for VAT — the same "not stated yet" default the
/// domain applies.
pub fn stubbed_organization() -> MockOrganizationRepository {
    let mut repo = MockOrganizationRepository::new();
    repo.expect_find_by_id().returning(move |id| {
        let now = Utc::now();
        let organization = Organization {
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
            field_clock_enabled: false,
            vat_on_debits: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        Box::pin(async move { Ok(Some(organization)) })
    });
    repo
}
