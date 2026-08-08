use common::CoreError;
use events::EventEnvelope;

/// Appends events to the durable log.
///
/// There is no read side and no update: the log is append-only, and the only
/// column that ever changes afterwards is `dispatched_at`, which belongs to
/// the dispatcher rather than to any emitter.
#[cfg_attr(test, mockall::automock)]
pub trait EventLogRepository: Send {
    /// Append a whole transaction's events in one statement.
    ///
    /// Called from inside the business transaction, immediately before it
    /// commits, so that a rollback takes the events with it.
    fn append(
        &mut self,
        events: &[EventEnvelope],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
