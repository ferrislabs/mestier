use std::sync::Mutex;

use common::CoreError;
use discord::{DomainEvent, EventPublisher, OrganizationId};

use super::{hub::EventHub, wire::from_domain};

/// Per-transaction event accumulator.
struct TxBuffer {
    events: Vec<DomainEvent>,
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
///
/// One publisher belongs to **one transaction of one organization**. Both are
/// fixed at construction, which is what makes a cross-organization delivery
/// unrepresentable rather than merely unlikely: there is no way to ask this
/// buffer to flush on behalf of somebody else. A single publisher shared by
/// every request is exactly how events used to reach the wrong tenant.
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
    /// the organization context for the current transaction; every buffered
    /// event in a transaction belongs to the same org.
    pub fn flush(&self, org_id: OrganizationId) {
        let events = {
            let mut buf = self.buffer.lock().unwrap();
            std::mem::take(&mut buf.events)
        };
        for domain_event in events {
            let wire = from_domain(domain_event, org_id);
            self.hub.broadcast(org_id, wire);
        }
    }
}

impl EventPublisher for RealtimeEventPublisher {
    async fn publish(&self, event: DomainEvent) -> Result<(), CoreError> {
        let mut buf = self
            .buffer
            .lock()
            .map_err(|_| CoreError::Internal("publisher buffer lock poisoned".into()))?;
        buf.events.push(event);
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
    use discord::{Category, CategoryId, ChannelId, MessageId};
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

    fn message_deleted() -> DomainEvent {
        DomainEvent::MessageDeleted {
            channel_id: ChannelId(Uuid::from_u128(10)),
            message_id: MessageId(Uuid::from_u128(11)),
        }
    }

    #[tokio::test]
    async fn buffered_events_not_emitted_before_flush() {
        let hub = EventHub::new();
        let o = org(1);
        let mut rx = hub.subscribe(o);
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
        let mut rx = hub.subscribe(o);
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

    /// The regression this whole change exists for.
    ///
    /// Production used to hold one `Arc<RealtimeEventPublisher>` on the
    /// long-lived `MestierUseCase`, so every request shared one buffer and the
    /// first transaction to commit drained everyone's events onto its own
    /// organization's channel — re-stamped with that organization's id.
    #[tokio::test]
    async fn one_transactions_events_are_invisible_to_anothers_flush() {
        let hub = EventHub::new();
        let org_a = org(100);
        let org_b = org(200);
        let mut rx_a = hub.subscribe(org_a);

        let tx_b = RealtimeEventPublisher::new(hub.clone());
        tx_b.publish(category_created(org_b)).await.unwrap();

        let tx_a = RealtimeEventPublisher::new(hub.clone());
        tx_a.flush(org_a);

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), rx_a.recv())
                .await
                .is_err(),
            "org A's flush must not drain org B's transaction"
        );
    }

    #[tokio::test]
    async fn dropped_buffer_emits_nothing() {
        let hub = EventHub::new();
        let o = org(3);
        let mut rx = hub.subscribe(o);

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

    #[tokio::test]
    async fn flush_populates_organization_id_for_message_delete() {
        let hub = EventHub::new();
        let o = org(4);
        let mut rx = hub.subscribe(o);
        let publisher = RealtimeEventPublisher::new(hub.clone());

        publisher.publish(message_deleted()).await.unwrap();
        publisher.flush(o);

        let received = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv())
            .await
            .expect("expected event within 50 ms")
            .unwrap();

        match received {
            crate::infrastructure::realtime::wire::GatewayEvent::MessageDelete {
                organization_id,
                ..
            } => {
                assert_eq!(organization_id, o, "organization_id must match flush org");
            }
            other => panic!("expected MessageDelete, got {:?}", other),
        }
    }
}
