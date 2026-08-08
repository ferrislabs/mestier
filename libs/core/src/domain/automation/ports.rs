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

/// What one pass of the fan-out did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DispatchOutcome {
    /// Events read and marked dispatched, including those nobody subscribed to.
    pub events: u64,
    /// Delivery rows created. Lower than `events` when subscriptions are
    /// sparse, higher when several subscribers want the same event.
    pub deliveries: u64,
}

/// Turns events into one delivery per interested subscriber.
///
/// Deliberately separate from executing a delivery: this does no external I/O,
/// so it cannot hang, and a subscriber whose endpoint is down cannot delay
/// anyone else's fan-out.
#[cfg_attr(test, mockall::automock)]
pub trait EventDispatchRepository: Send {
    fn dispatch_pending(
        &mut self,
        batch: i64,
    ) -> impl Future<Output = Result<DispatchOutcome, CoreError>> + Send;
}
