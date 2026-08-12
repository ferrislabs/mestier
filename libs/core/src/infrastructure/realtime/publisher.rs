use std::sync::{Arc, Mutex};

use common::CoreError;
use discord::{DomainEvent, EventPublisher};

use super::{hub::EventHub, wire::from_domain};

/// Per-transaction event accumulator.
///
/// Owned by the caller *outside* the transaction closure and shared with the
/// [`RealtimeEventPublisher`] built inside it. That split is the whole point:
/// the publisher must live and die with the transaction, but the flush has to
/// happen after the commit, so the buffer has to outlive the closure while
/// still belonging to exactly one transaction.
#[derive(Clone, Default)]
pub struct RealtimeBuffer {
    events: Arc<Mutex<Vec<DomainEvent>>>,
}

impl RealtimeBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drain the buffer and broadcast each event on its own organization's channel.
    ///
    /// Must be called **after** the enclosing transaction commits — and now it
    /// structurally can be, because the buffer outlives the closure. Emitting
    /// before the commit meant a subscriber could observe a message whose
    /// transaction went on to fail.
    ///
    /// Each event names its own organization, so a buffer holding events for
    /// two organizations delivers each to the right one. There is no org
    /// argument to get wrong.
    pub fn flush(&self, hub: &EventHub) {
        let events = {
            let Ok(mut buf) = self.events.lock() else {
                // A poisoned buffer means a panic happened mid-transaction.
                // Dropping the events is correct: that transaction did not commit.
                return;
            };
            std::mem::take(&mut *buf)
        };
        for domain_event in events {
            let wire = from_domain(domain_event);
            hub.broadcast(wire.organization_id(), wire);
        }
    }
}

/// Implements `discord::EventPublisher` by buffering into a [`RealtimeBuffer`].
///
/// Built for **one transaction** and never shared between two: a single
/// publisher held on the long-lived use case is exactly how one request's
/// commit used to drain another's events.
pub struct RealtimeEventPublisher {
    buffer: RealtimeBuffer,
}

impl RealtimeEventPublisher {
    pub fn new(buffer: RealtimeBuffer) -> Self {
        Self { buffer }
    }
}

impl EventPublisher for RealtimeEventPublisher {
    async fn publish(&self, event: DomainEvent) -> Result<(), CoreError> {
        let mut buf = self
            .buffer
            .events
            .lock()
            .map_err(|_| CoreError::Internal("publisher buffer lock poisoned".into()))?;
        buf.push(event);
        Ok(())
    }
}

// Services are generic over `E: EventPublisher`, and the use case passes
// `&events` — the publisher `#[transactional(events)]` built for this
// transaction. This impl is what lets the reference satisfy the bound.
impl EventPublisher for &RealtimeEventPublisher {
    async fn publish(&self, event: DomainEvent) -> Result<(), CoreError> {
        (*self).publish(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::realtime::wire::GatewayEvent;
    use discord::{Category, CategoryId, ChannelId, MessageId, OrganizationId};
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

    fn message_deleted(org_id: OrganizationId) -> DomainEvent {
        DomainEvent::MessageDeleted {
            organization_id: org_id,
            channel_id: ChannelId(Uuid::from_u128(10)),
            message_id: MessageId(Uuid::from_u128(11)),
        }
    }

    #[tokio::test]
    async fn buffered_events_not_emitted_before_flush() {
        let hub = EventHub::new();
        let o = org(1);
        let mut rx = hub.subscribe(o);
        let buffer = RealtimeBuffer::new();
        let publisher = RealtimeEventPublisher::new(buffer.clone());

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
        let mut rx = hub.subscribe(o);
        let buffer = RealtimeBuffer::new();
        let publisher = RealtimeEventPublisher::new(buffer.clone());

        publisher.publish(category_created(o)).await.unwrap();
        buffer.flush(&hub);

        let received = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv())
            .await
            .expect("expected event within 50 ms")
            .unwrap();

        assert!(matches!(received, GatewayEvent::CategoryCreate(_)));
    }

    /// The regression this whole change exists for.
    ///
    /// The org used to be a `flush(org_id)` argument re-stamped onto every
    /// buffered event, so a buffer could only ever deliver to one organization
    /// — and delivering to the wrong one was a single mistyped argument away.
    /// Now each event names its own, and a mixed buffer routes correctly.
    #[tokio::test]
    async fn flush_routes_each_event_to_its_own_organization() {
        let hub = EventHub::new();
        let org_a = org(100);
        let org_b = org(200);
        let mut rx_a = hub.subscribe(org_a);
        let mut rx_b = hub.subscribe(org_b);

        let buffer = RealtimeBuffer::new();
        let publisher = RealtimeEventPublisher::new(buffer.clone());
        publisher.publish(category_created(org_a)).await.unwrap();
        publisher.publish(message_deleted(org_b)).await.unwrap();

        buffer.flush(&hub);

        let a = tokio::time::timeout(std::time::Duration::from_millis(50), rx_a.recv())
            .await
            .expect("org A must receive its event")
            .unwrap();
        assert_eq!(a.organization_id(), org_a);
        assert!(matches!(a, GatewayEvent::CategoryCreate(_)));

        let b = tokio::time::timeout(std::time::Duration::from_millis(50), rx_b.recv())
            .await
            .expect("org B must receive its event")
            .unwrap();
        assert_eq!(b.organization_id(), org_b);
        assert!(matches!(b, GatewayEvent::MessageDelete { .. }));

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), rx_a.recv())
                .await
                .is_err(),
            "org A must not also receive org B's event"
        );
    }

    #[tokio::test]
    async fn dropped_buffer_emits_nothing() {
        let hub = EventHub::new();
        let o = org(3);
        let mut rx = hub.subscribe(o);

        {
            let buffer = RealtimeBuffer::new();
            let publisher = RealtimeEventPublisher::new(buffer.clone());
            publisher.publish(category_created(o)).await.unwrap();
            // buffer and publisher dropped without flush — simulates a rolled-back tx
        }

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), rx.recv())
                .await
                .is_err(),
            "a buffer dropped without flush must emit nothing"
        );
    }

    #[tokio::test]
    async fn message_delete_carries_its_own_organization() {
        let hub = EventHub::new();
        let o = org(4);
        let mut rx = hub.subscribe(o);
        let buffer = RealtimeBuffer::new();
        let publisher = RealtimeEventPublisher::new(buffer.clone());

        publisher.publish(message_deleted(o)).await.unwrap();
        buffer.flush(&hub);

        let received = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv())
            .await
            .expect("expected event within 50 ms")
            .unwrap();

        match received {
            GatewayEvent::MessageDelete {
                organization_id, ..
            } => assert_eq!(organization_id, o),
            other => panic!("expected MessageDelete, got {other:?}"),
        }
    }
}
