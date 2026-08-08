use std::sync::Mutex;

use common::{CoreError, OrganizationId};
use events::{Actor, DomainEvent, EmissionContext, EventEnvelope};
use uuid::Uuid;

/// Accumulates the events of **one transaction**, in memory, until the
/// transaction is ready to persist them.
///
/// One emitter belongs to one transaction. It is built by
/// `#[transactional]` and never stored on the use case — a shared buffer is
/// how realtime events used to reach the wrong organization (see the history
/// of `RealtimeEventPublisher`).
///
/// The organization is passed per event rather than fixed at construction:
/// the `update_*` use cases identify their aggregate by id and only learn
/// which organization it belongs to once the service has loaded it.
pub struct TransactionalEventEmitter {
    actor: Actor,
    correlation_id: Option<Uuid>,
    buffer: Mutex<Vec<EventEnvelope>>,
}

impl TransactionalEventEmitter {
    pub fn new(actor: Actor, correlation_id: Option<Uuid>) -> Self {
        Self {
            actor,
            correlation_id,
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub fn emit<E: DomainEvent>(&self, org_id: OrganizationId, event: &E) -> Result<(), CoreError> {
        let envelope = EventEnvelope::from_event(
            event,
            &EmissionContext {
                org_id,
                actor: self.actor,
                correlation_id: self.correlation_id,
            },
        );

        self.buffer
            .lock()
            .map_err(|_| CoreError::Internal("event emitter buffer lock poisoned".into()))?
            .push(envelope);

        Ok(())
    }

    /// Take everything buffered so far, leaving the emitter empty.
    pub fn drain(&self) -> Result<Vec<EventEnvelope>, CoreError> {
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|_| CoreError::Internal("event emitter buffer lock poisoned".into()))?;

        Ok(std::mem::take(&mut buffer))
    }
}

#[cfg(test)]
mod tests {
    use events::EventSubject;
    use serde_json::{Value, json};

    use super::*;

    struct QuoteAccepted {
        quote_id: Uuid,
    }

    impl DomainEvent for QuoteAccepted {
        fn name(&self) -> &'static str {
            "quote.accepted"
        }

        fn version(&self) -> u16 {
            1
        }

        fn subject(&self) -> EventSubject {
            EventSubject::new("quote", self.quote_id)
        }

        fn payload(&self) -> Value {
            json!({ "quote_id": self.quote_id })
        }
    }

    fn org(n: u128) -> OrganizationId {
        OrganizationId(Uuid::from_u128(n))
    }

    #[test]
    fn drain_returns_what_was_emitted_in_order() {
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);

        emitter
            .emit(
                org(1),
                &QuoteAccepted {
                    quote_id: Uuid::from_u128(10),
                },
            )
            .unwrap();
        emitter
            .emit(
                org(1),
                &QuoteAccepted {
                    quote_id: Uuid::from_u128(11),
                },
            )
            .unwrap();

        let drained = emitter.drain().unwrap();

        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].subject_id, Some(Uuid::from_u128(10)));
        assert_eq!(drained[1].subject_id, Some(Uuid::from_u128(11)));
    }

    #[test]
    fn draining_twice_yields_nothing_the_second_time() {
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);
        emitter
            .emit(
                org(1),
                &QuoteAccepted {
                    quote_id: Uuid::from_u128(10),
                },
            )
            .unwrap();

        emitter.drain().unwrap();

        assert!(emitter.drain().unwrap().is_empty());
    }

    #[test]
    fn an_envelope_carries_the_emitters_actor_and_correlation() {
        let user_id = Uuid::from_u128(7);
        let correlation_id = Uuid::from_u128(8);
        let emitter = TransactionalEventEmitter::new(Actor::user(user_id), Some(correlation_id));

        emitter
            .emit(
                org(1),
                &QuoteAccepted {
                    quote_id: Uuid::from_u128(10),
                },
            )
            .unwrap();

        let drained = emitter.drain().unwrap();

        assert_eq!(drained[0].actor, Actor::user(user_id));
        assert_eq!(drained[0].correlation_id, Some(correlation_id));
    }

    /// Two aggregates of two organizations can be written by one transaction —
    /// rare, but the emitter must not silently attribute them to one org.
    #[test]
    fn each_envelope_keeps_the_organization_it_was_emitted_for() {
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);

        emitter
            .emit(
                org(1),
                &QuoteAccepted {
                    quote_id: Uuid::from_u128(10),
                },
            )
            .unwrap();
        emitter
            .emit(
                org(2),
                &QuoteAccepted {
                    quote_id: Uuid::from_u128(11),
                },
            )
            .unwrap();

        let drained = emitter.drain().unwrap();

        assert_eq!(drained[0].org_id, org(1));
        assert_eq!(drained[1].org_id, org(2));
    }
}
