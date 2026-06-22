use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use discord::OrganizationId;
use tokio::sync::broadcast::{self, Receiver, Sender};

use super::wire::GatewayEvent;

const HUB_CHANNEL_CAPACITY: usize = 256;

struct HubInner {
    /// One broadcast channel per organization — guarantees per-org delivery isolation.
    senders: HashMap<OrganizationId, Sender<GatewayEvent>>,
}

/// In-process event bus.  Clone-cheap: all clones share the same `Arc<Mutex<HubInner>>`.
#[derive(Clone)]
pub struct EventHub {
    inner: Arc<Mutex<HubInner>>,
}

impl EventHub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HubInner {
                senders: HashMap::new(),
            })),
        }
    }

    /// Subscribe to events for `orgs[0]`.  Creates the per-org channel on first use.
    /// Callers that belong to multiple orgs must call `subscribe` once per org and
    /// select across all receivers in their WebSocket task.
    pub fn subscribe(&self, orgs: &[OrganizationId]) -> Receiver<GatewayEvent> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(first) = orgs.first() {
            let tx = inner
                .senders
                .entry(*first)
                .or_insert_with(|| broadcast::channel(HUB_CHANNEL_CAPACITY).0);
            tx.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(1);
            drop(tx);
            rx
        }
    }

    /// Broadcast `event` to every subscriber registered for `org`.
    /// If there are no current subscribers the send is silently dropped.
    pub fn broadcast(&self, org: OrganizationId, event: GatewayEvent) {
        let mut inner = self.inner.lock().unwrap();
        let tx = inner
            .senders
            .entry(org)
            .or_insert_with(|| broadcast::channel(HUB_CHANNEL_CAPACITY).0);
        let _ = tx.send(event);
    }
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use discord::{ChannelId, MessageId};
    use uuid::Uuid;

    fn org(n: u128) -> OrganizationId {
        OrganizationId(Uuid::from_u128(n))
    }

    fn delete_event() -> GatewayEvent {
        GatewayEvent::MessageDelete {
            channel_id: ChannelId(Uuid::from_u128(2)),
            message_id: MessageId(Uuid::from_u128(3)),
        }
    }

    #[tokio::test]
    async fn broadcast_fan_out_to_subscriber_for_matching_org() {
        let hub = EventHub::new();
        let o = org(1);
        let mut rx = hub.subscribe(&[o]);

        hub.broadcast(o, delete_event());

        let received = rx.recv().await.unwrap();
        assert!(matches!(received, GatewayEvent::MessageDelete { .. }));
    }

    #[tokio::test]
    async fn broadcast_not_received_by_different_org_subscriber() {
        let hub = EventHub::new();
        let o_a = org(10);
        let o_b = org(20);
        let mut rx = hub.subscribe(&[o_a]);

        hub.broadcast(o_b, delete_event());

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), rx.recv())
                .await
                .is_err(),
            "org-A subscriber must not receive an org-B event"
        );
    }
}
