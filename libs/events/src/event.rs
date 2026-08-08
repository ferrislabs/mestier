use serde_json::Value;
use uuid::Uuid;

/// What an event is about: the aggregate kind and, when there is one, its id.
///
/// `id` is optional because an event can concern an organization as a whole
/// rather than a single aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSubject {
    pub kind: &'static str,
    pub id: Option<Uuid>,
}

impl EventSubject {
    pub fn new(kind: &'static str, id: Uuid) -> Self {
        Self { kind, id: Some(id) }
    }

    pub fn organization_wide(kind: &'static str) -> Self {
        Self { kind, id: None }
    }
}

/// A business fact a module chose to publish.
///
/// Implementors are declared by the module that owns the aggregate, never by
/// this crate: the backbone knows how to carry an event, not what events exist.
///
/// Deliberately **not** object-safe-by-design: the emitter is generic over `E`
/// and converts to [`crate::EventEnvelope`] the moment `emit` is called, so a
/// transaction buffers concrete envelopes rather than trait objects.
pub trait DomainEvent {
    /// Dotted, stable, and part of the public contract: `quote.accepted`.
    fn name(&self) -> &'static str;

    /// Incremented when the payload changes incompatibly. Never reused.
    fn version(&self) -> u16;

    fn subject(&self) -> EventSubject;

    /// The serialized domain model — never the database row.
    fn payload(&self) -> Value;
}
