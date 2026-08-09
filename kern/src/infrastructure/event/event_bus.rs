use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::{broadcast, watch};

use crate::{
    application::event::{EventBus, EventPublisher},
    building_blocks::domain_event::DynDomainEvent,
};

/// The TokioEventBus is the orchestrator between tokio's broadcast::Senders and the EventHandlers.
/// It passes the domain events to the broadcast::Senders to send to EventHandlers which act on the
/// domain events in a background process
pub struct TokioEventBus {
    publishers: DashMap<&'static str, broadcast::Sender<Arc<dyn DynDomainEvent>>>,
    shutdown_tx: watch::Sender<bool>,
}

impl TokioEventBus {
    /// Creates a new instance of the TokioEventBus
    pub fn new() -> Self {
        let (tx, _) = watch::channel(false);
        Self {
            publishers: DashMap::new(),
            shutdown_tx: tx,
        }
    }

    /// Sends a shutdown signal to all handlers
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Retrives the publisher for the topic or creates one if it doesn't exist yet for that topic
    /// # Arguments
    /// * `topic` - The topic the publisher will send domain events to
    fn get_publisher(&self, topic: &'static str) -> broadcast::Sender<Arc<dyn DynDomainEvent>> {
        let publisher = self.publishers.entry(topic).or_insert_with(|| {
            let (tx, _) = broadcast::channel(1024);
            tx
        });
        publisher.value().clone()
    }
}

impl Default for TokioEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisher for TokioEventBus {
    fn publish(&self, topic: &'static str, event: Arc<dyn DynDomainEvent>) {
        let tx = self.get_publisher(topic);
        let _ = tx.send(event);
    }
}

impl EventBus for TokioEventBus {
    fn register_handler(
        &self,
        topic: &'static str,
        handler: Box<dyn crate::application::event::EventHandler>,
    ) {
        let mut rx = self.get_publisher(topic).subscribe();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() { break; }
                    }

                    result = rx.recv() => {
                        match result {
                            Ok(event) => handler.handle(event).await,
                            Err(_) => break,
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::event::EventHandler;
    use crate::building_blocks::{
        domain_event::{DomainEvent, DynDomainEvent},
        ids::{AggregateId, EventId},
    };
    use chrono::{DateTime, Utc};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;
    use uuid::Uuid;

    #[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
    pub struct AccountId {
        pub id: Uuid,
    }
    impl AggregateId for AccountId {}

    #[derive(Debug)]
    pub struct CreatedAccount {
        id: EventId,
        aggregate_id: AccountId,
        aggregate_version: u32,
        occurred_on: DateTime<Utc>,
    }

    impl CreatedAccount {
        pub fn new(aggregate_id: uuid::Uuid) -> Self {
            Self {
                id: EventId::new_random_v4(),
                aggregate_id: AccountId { id: aggregate_id },
                aggregate_version: 0,
                occurred_on: Utc::now(),
            }
        }
    }

    impl DomainEvent for CreatedAccount {
        type Id = AccountId;
        fn id(&self) -> &EventId {
            &self.id
        }
        fn aggregate_id(&self) -> &AccountId {
            &self.aggregate_id
        }
        fn aggregate_version(&self) -> u32 {
            self.aggregate_version
        }
        fn occurred_on(&self) -> &DateTime<Utc> {
            &self.occurred_on
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn event_type(&self) -> &'static str {
            "created-account"
        }
    }

    struct AccountHandler {
        count: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl EventHandler for AccountHandler {
        async fn handle(&self, event: Arc<dyn DynDomainEvent>) {
            // Logic to verify the event type
            if let Some(_e) = event.as_any().downcast_ref::<CreatedAccount>() {
                self.count.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    #[tokio::test]
    async fn given_a_handler_when_publishing_event_then_subscriber_receives_it() {
        // Arrange
        let bus = TokioEventBus::new();
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        // Register handlers
        bus.register_handler(
            "account-created",
            Box::new(AccountHandler {
                count: count1.clone(),
            }),
        );
        bus.register_handler(
            "account-updated",
            Box::new(AccountHandler {
                count: count2.clone(),
            }),
        );

        // Act
        let event = CreatedAccount::new(Uuid::new_v4());
        let event2 = CreatedAccount::new(Uuid::new_v4());

        bus.publish("account-created", Arc::new(event));
        bus.publish("account-created", Arc::new(event2));

        // Assert - We must yield or wait slightly because the handler is async/spawned
        // A small sleep ensures the background tasks have time to process
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(
            count1.load(Ordering::SeqCst),
            2,
            "Topic 'account-created' should have processed 2 events"
        );
        assert_eq!(
            count2.load(Ordering::SeqCst),
            0,
            "Topic 'account-updated' should have processed 0 events"
        );
    }

    #[tokio::test]
    async fn test_shutdown_stops_listeners() {
        let bus = TokioEventBus::new();
        let count = Arc::new(AtomicUsize::new(0));

        bus.register_handler(
            "test",
            Box::new(AccountHandler {
                count: count.clone(),
            }),
        );

        // Kill the bus
        bus.shutdown();
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Publish after shutdown
        bus.publish("test", Arc::new(CreatedAccount::new(Uuid::new_v4())));
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Count should remain 0 because the loop broke
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }
}
