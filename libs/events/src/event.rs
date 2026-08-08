use common::{CoreError, OrganizationId};
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

/// Port a domain service uses to publish what it just did.
///
/// Generic rather than object-safe: the implementation converts to an
/// [`crate::EventEnvelope`] at call time, so nothing needs to buffer trait
/// objects. `libs/core` implements it once, on the emitter `#[transactional]`
/// builds for each transaction.
///
/// The organization is a parameter rather than emitter state because the
/// `update_*` use cases identify their aggregate by id and only learn which
/// organization owns it once the service has loaded it.
pub trait EventEmitter {
    fn emit<E: DomainEvent>(&self, org_id: OrganizationId, event: &E) -> Result<(), CoreError>;
}
