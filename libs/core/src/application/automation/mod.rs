use common::CoreError;
use mestier_macros::transactional;

use crate::{
    application::MestierUseCase,
    domain::automation::ports::{DispatchOutcome, EventDispatchRepository},
};

mod credential;
pub mod retention;
pub mod run;
mod subscription;
mod tests;
mod workflow;

impl MestierUseCase {
    /// Runs one fan-out pass: read undispatched events, create a workflow run
    /// per interested subscription, mark them done.
    ///
    /// Bounded by `batch` and safe to run concurrently — the claim uses
    /// `FOR UPDATE SKIP LOCKED`, so a second caller picks up other events
    /// rather than waiting.
    #[transactional(event_dispatch)]
    pub async fn dispatch_pending_events(&self, batch: i64) -> Result<DispatchOutcome, CoreError> {
        let mut repository = event_dispatch_repository;
        repository.dispatch_pending(batch).await
    }
}
