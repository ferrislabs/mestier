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

use common::CoreError;
use mestier_core::{MockQuoteRepository, Quote};

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

    // Same `DEV-<year>-NNNN` shape as the Postgres adapter, and derived from
    // what the store holds, so the scenario stating a clean year is what
    // produces `0001` rather than the stub asserting its own constant.
    let store_ref = store.clone();
    repo.expect_next_reference().returning(move |org, year| {
        let prefix = format!("DEV-{year}-");
        let count = quotes(&store_ref)
            .iter()
            .filter(|quote| quote.organization_id == org && quote.reference.starts_with(&prefix))
            .count();

        Box::pin(async move { Ok(format!("{prefix}{:04}", count + 1)) })
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
        .returning(move |id, status, updated_at| {
            let outcome = match quotes(&store_ref).iter_mut().find(|quote| quote.id == id) {
                Some(quote) => {
                    quote.status = status;
                    quote.updated_at = updated_at;
                    Ok(quote.clone())
                }
                None => Err(CoreError::NotFound),
            };

            Box::pin(async move { outcome })
        });

    repo
}
