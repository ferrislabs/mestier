use std::sync::Mutex;

use common::CoreError;
use discord::{DomainEvent, EventPublisher, OrganizationId};

use super::{hub::EventHub, wire::from_domain};

/// Per-transaction event accumulator.
struct TxBuffer {
    events: Vec<(OrganizationId, DomainEvent)>,
}

impl TxBuffer {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

/// Implements `discord::EventPublisher`.
///
/// Events are **buffered** via `publish` during a database transaction and only
/// forwarded to the [`EventHub`] when `flush` is called after the transaction
/// commits successfully.  If the transaction rolls back the buffer is simply
/// dropped, emitting nothing.
pub struct RealtimeEventPublisher {
    hub: EventHub,
    buffer: Mutex<TxBuffer>,
}

impl RealtimeEventPublisher {
    pub fn new(hub: EventHub) -> Self {
        Self {
            hub,
            buffer: Mutex::new(TxBuffer::new()),
        }
    }

    /// Drain the buffer and broadcast each event to the hub.
    ///
    /// Must be called **after** the enclosing transaction commits.  `org_id` is
    /// the organization context for the current transaction; it is used as the
    /// broadcast key for events that do not carry an `organization_id` inline
    /// (reactions, message deletes).
    pub fn flush(&self, org_id: OrganizationId) {
        let events = {
            let mut buf = self.buffer.lock().unwrap();
            std::mem::take(&mut buf.events)
        };
        for (event_org, domain_event) in events {
            let wire = from_domain(domain_event, event_org);
            self.hub.broadcast(org_id, wire);
        }
    }
}

impl EventPublisher for RealtimeEventPublisher {
    async fn publish(&self, event: DomainEvent) -> Result<(), CoreError> {
        let org_id = org_id_from_event(&event);
        let mut buf = self
            .buffer
            .lock()
            .map_err(|_| CoreError::Internal("publisher buffer lock poisoned".into()))?;
        buf.events.push((org_id, event));
        Ok(())
    }
}

/// Extract the `organization_id` from events that carry one inline.
/// For events without an inline org (reactions, message deletes) a nil UUID is
/// stored; `flush(org_id)` supplies the correct org from the call site.
fn org_id_from_event(event: &DomainEvent) -> OrganizationId {
    match event {
        DomainEvent::MessageCreated(m) => m.organization_id,
        DomainEvent::MessageUpdated(m) => m.organization_id,
        DomainEvent::MessageDeleted { .. } => OrganizationId(uuid::Uuid::nil()),
        DomainEvent::ReactionAdded { .. } => OrganizationId(uuid::Uuid::nil()),
        DomainEvent::ReactionRemoved { .. } => OrganizationId(uuid::Uuid::nil()),
        DomainEvent::CategoryCreated(c) => c.organization_id,
        DomainEvent::CategoryUpdated(c) => c.organization_id,
        DomainEvent::CategoryDeleted {
            organization_id, ..
        } => *organization_id,
        DomainEvent::ChannelCreated(c) => c.organization_id,
        DomainEvent::ChannelUpdated(c) => c.organization_id,
        DomainEvent::ChannelDeleted {
            organization_id, ..
        } => *organization_id,
        DomainEvent::ThreadCreated(c) => c.organization_id,
        DomainEvent::ThreadUpdated(c) => c.organization_id,
        DomainEvent::ThreadDeleted {
            organization_id, ..
        } => *organization_id,
        DomainEvent::PresenceUpdated {
            organization_id, ..
        } => *organization_id,
        DomainEvent::TypingStarted {
            organization_id, ..
        } => *organization_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use discord::{Category, CategoryId};
    use uuid::Uuid;

    fn org(n: u128) -> OrganizationId {
        OrganizationId(Uuid::from_u128(n))
    }

    fn category_created(org_id: OrganizationId) -> DomainEvent {
        DomainEvent::CategoryCreated(Category {
            id: CategoryId(Uuid::from_u128(99)),
            organization_id: org_id,
            name: "test".into(),
            position: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    #[tokio::test]
    async fn buffered_events_not_emitted_before_flush() {
        let hub = EventHub::new();
        let o = org(1);
        let mut rx = hub.subscribe(&[o]);
        let publisher = RealtimeEventPublisher::new(hub.clone());

        publisher.publish(category_created(o)).await.unwrap();

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), rx.recv())
                .await
                .is_err(),
            "event must not reach the channel before flush"
        );
    }

    #[tokio::test]
    async fn flush_emits_event_to_subscriber() {
        let hub = EventHub::new();
        let o = org(2);
        let mut rx = hub.subscribe(&[o]);
        let publisher = RealtimeEventPublisher::new(hub.clone());

        publisher.publish(category_created(o)).await.unwrap();
        publisher.flush(o);

        let received = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv())
            .await
            .expect("expected event within 50 ms")
            .unwrap();

        assert!(
            matches!(
                received,
                crate::infrastructure::realtime::wire::GatewayEvent::CategoryCreate(_)
            ),
            "expected CategoryCreate wire event"
        );
    }

    #[tokio::test]
    async fn dropped_buffer_emits_nothing() {
        let hub = EventHub::new();
        let o = org(3);
        let mut rx = hub.subscribe(&[o]);

        {
            let publisher = RealtimeEventPublisher::new(hub.clone());
            publisher.publish(category_created(o)).await.unwrap();
            // publisher dropped here without flush — simulates a rolled-back tx
        }

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), rx.recv())
                .await
                .is_err(),
            "dropped publisher must not emit events"
        );
    }
}
