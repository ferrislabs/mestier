use chrono::{DateTime, Utc};
use common::{OrganizationId, generate_uuid_v7};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{Actor, DomainEvent};

/// What a transaction knows that an event does not: which organization it
/// happened in, who caused it, and which request it belongs to.
#[derive(Debug, Clone, Copy)]
pub struct EmissionContext {
    pub org_id: OrganizationId,
    pub actor: Actor,
    pub correlation_id: Option<Uuid>,
}

/// The persisted form of an event: one row of the log, one payload on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub name: String,
    pub version: u16,
    pub subject_kind: String,
    pub subject_id: Option<Uuid>,
    pub payload: Value,
    pub actor: Actor,
    pub correlation_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
}

impl EventEnvelope {
    pub fn from_event<E: DomainEvent>(event: &E, context: &EmissionContext) -> Self {
        let subject = event.subject();

        Self {
            // v7 is time-ordered, so the log's primary key already sorts the
            // way the dispatcher reads it.
            id: generate_uuid_v7(),
            org_id: context.org_id,
            name: event.name().to_owned(),
            version: event.version(),
            subject_kind: subject.kind.to_owned(),
            subject_id: subject.id,
            payload: event.payload(),
            actor: context.actor,
            correlation_id: context.correlation_id,
            occurred_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::EventSubject;

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

    #[test]
    fn envelope_carries_the_event_identity_and_its_emission_context() {
        let quote_id = Uuid::from_u128(1);
        let org_id = OrganizationId(Uuid::from_u128(2));
        let user_id = Uuid::from_u128(3);
        let correlation_id = Uuid::from_u128(4);

        let envelope = EventEnvelope::from_event(
            &QuoteAccepted { quote_id },
            &EmissionContext {
                org_id,
                actor: Actor::user(user_id),
                correlation_id: Some(correlation_id),
            },
        );

        assert_eq!(envelope.name, "quote.accepted");
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.subject_kind, "quote");
        assert_eq!(envelope.subject_id, Some(quote_id));
        assert_eq!(envelope.payload, json!({ "quote_id": quote_id }));
        assert_eq!(envelope.org_id, org_id);
        assert_eq!(envelope.actor, Actor::user(user_id));
        assert_eq!(envelope.correlation_id, Some(correlation_id));
    }
}
