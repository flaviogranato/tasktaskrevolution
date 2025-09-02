use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid7::Uuid;

/// A domain event that can be observed
#[allow(dead_code)]
pub trait DomainEvent: Send + Sync {
    /// Get the event type
    fn event_type(&self) -> &str;

    /// Get the event ID
    fn event_id(&self) -> Uuid;

    /// Get the timestamp when the event occurred
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the aggregate ID that generated this event
    fn aggregate_id(&self) -> &str;

    /// Get the event version
    fn version(&self) -> u64;

    /// Get the event data as YAML
    fn data(&self) -> serde_yaml::Value;
}

/// An observer that can handle domain events
#[allow(dead_code)]
pub trait EventObserver: Send + Sync {
    /// Handle a domain event
    fn handle_event(&self, event: &dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get the observer name
    fn name(&self) -> &str;

    /// Check if the observer is interested in a specific event type
    fn is_interested_in(&self, _event_type: &str) -> bool {
        true
    }
}

/// An event bus that manages event publishing and subscription
#[allow(dead_code)]
pub struct EventBus {
    observers: Arc<Mutex<ObserverMap>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe an observer to a specific event type
    pub fn subscribe(&self, event_type: impl Into<String>, observer: Arc<dyn EventObserver>) {
        let mut observers = self.observers.lock().unwrap();
        observers.entry(event_type.into()).or_default().push(observer);
    }

    /// Unsubscribe an observer from a specific event type
    pub fn unsubscribe(&self, event_type: &str, observer_name: &str) {
        let mut observers = self.observers.lock().unwrap();
        if let Some(observer_list) = observers.get_mut(event_type) {
            observer_list.retain(|obs| obs.name() != observer_name);
        }
    }

    /// Publish an event to all interested observers
    pub fn publish(&self, event: &dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_type = event.event_type();
        let observers = self.observers.lock().unwrap();

        if let Some(observer_list) = observers.get(event_type) {
            for observer in observer_list {
                if observer.is_interested_in(event_type)
                    && let Err(e) = observer.handle_event(event)
                {
                    eprintln!("Observer {} failed to handle event: {}", observer.name(), e);
                }
            }
        }

        Ok(())
    }

    /// Publish an event asynchronously
    pub fn publish_async(&self, event: Box<dyn DomainEvent + Send>) {
        let event_bus = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = event_bus.publish(event.as_ref()) {
                eprintln!("Failed to publish event asynchronously: {}", e);
            }
        });
    }

    /// Get the number of observers for a specific event type
    pub fn observer_count(&self, event_type: &str) -> usize {
        let observers = self.observers.lock().unwrap();
        observers.get(event_type).map_or(0, |list| list.len())
    }

    /// Get all event types that have observers
    pub fn event_types(&self) -> Vec<String> {
        let observers = self.observers.lock().unwrap();
        observers.keys().cloned().collect()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            observers: Arc::clone(&self.observers),
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple event implementation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SimpleDomainEvent {
    pub event_type: String,
    pub event_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub aggregate_id: String,
    pub version: u64,
    pub data: serde_yaml::Value,
}

impl SimpleDomainEvent {
    /// Create a new simple domain event
    pub fn new(event_type: impl Into<String>, aggregate_id: impl Into<String>, data: serde_yaml::Value) -> Self {
        Self {
            event_type: event_type.into(),
            event_id: uuid7::uuid7(),
            timestamp: chrono::Utc::now(),
            aggregate_id: aggregate_id.into(),
            version: 1,
            data,
        }
    }

    /// Set the event version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }

    /// Set the event timestamp
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

impl DomainEvent for SimpleDomainEvent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn data(&self) -> serde_yaml::Value {
        self.data.clone()
    }
}

// Type aliases to reduce complexity
type ObserverList = Vec<Arc<dyn EventObserver>>;
type ObserverMap = HashMap<String, ObserverList>;
type EventHandler = Box<dyn Fn(&dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;

/// A simple observer implementation
#[allow(dead_code)]
pub struct SimpleEventObserver {
    name: String,
    event_types: Vec<String>,
    handler: EventHandler,
}

impl SimpleEventObserver {
    /// Create a new simple event observer
    pub fn new<F>(name: impl Into<String>, event_types: Vec<String>, handler: F) -> Self
    where
        F: for<'a> Fn(&'a dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static, // 'static necessário para Box<dyn>
    {
        Self {
            name: name.into(),
            event_types,
            handler: Box::new(handler),
        }
    }

    /// Create an observer that handles all event types
    pub fn new_universal<F>(name: impl Into<String>, handler: F) -> Self
    where
        F: for<'a> Fn(&'a dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static, // 'static necessário para Box<dyn>
    {
        Self {
            name: name.into(),
            event_types: Vec::new(),
            handler: Box::new(handler),
        }
    }
}

impl EventObserver for SimpleEventObserver {
    fn handle_event(&self, event: &dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self.handler)(event)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_interested_in(&self, event_type: &str) -> bool {
        self.event_types.is_empty() || self.event_types.contains(&event_type.to_string())
    }
}

/// An event store that can persist events
#[allow(dead_code)]
pub trait EventStore {
    /// Store an event
    fn store_event(&self, event: &dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Retrieve events for a specific aggregate
    fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<Box<dyn DomainEvent + Send>>, Box<dyn std::error::Error + Send + Sync>>;

    /// Retrieve events of a specific type
    fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<Box<dyn DomainEvent + Send>>, Box<dyn std::error::Error + Send + Sync>>;
}

/// An event replay mechanism
#[allow(dead_code)]
pub struct EventReplayer {
    event_store: Box<dyn EventStore>,
    event_bus: EventBus,
}

impl EventReplayer {
    /// Create a new event replayer
    pub fn new(event_store: Box<dyn EventStore>, event_bus: EventBus) -> Self {
        Self { event_store, event_bus }
    }

    /// Replay all events for a specific aggregate
    pub fn replay_aggregate_events(&self, aggregate_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let events = self.event_store.get_events(aggregate_id)?;

        for event in events {
            self.event_bus.publish(event.as_ref())?;
        }

        Ok(())
    }

    /// Replay all events of a specific type
    pub fn replay_events_by_type(&self, event_type: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let events = self.event_store.get_events_by_type(event_type)?;

        for event in events {
            self.event_bus.publish(event.as_ref())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock event store for testing
    struct MockEventStore {
        events: Vec<Box<dyn DomainEvent + Send>>,
    }

    impl MockEventStore {
        fn new() -> Self {
            Self { events: Vec::new() }
        }

        fn add_event(&mut self, event: Box<dyn DomainEvent + Send>) {
            self.events.push(event);
        }
    }

    impl EventStore for MockEventStore {
        fn store_event(&self, _event: &dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }

        fn get_events(
            &self,
            aggregate_id: &str,
        ) -> Result<Vec<Box<dyn DomainEvent + Send>>, Box<dyn std::error::Error + Send + Sync>> {
            let filtered_events: Vec<Box<dyn DomainEvent + Send>> = self
                .events
                .iter()
                .filter(|event| event.aggregate_id() == aggregate_id)
                .map(|event| {
                    // Create a new SimpleDomainEvent with the same data
                    let simple_event = SimpleDomainEvent {
                        event_type: event.event_type().to_string(),
                        event_id: event.event_id(),
                        timestamp: event.timestamp(),
                        aggregate_id: event.aggregate_id().to_string(),
                        version: event.version(),
                        data: event.data().clone(),
                    };
                    Box::new(simple_event) as Box<dyn DomainEvent + Send>
                })
                .collect();
            Ok(filtered_events)
        }

        fn get_events_by_type(
            &self,
            event_type: &str,
        ) -> Result<Vec<Box<dyn DomainEvent + Send>>, Box<dyn std::error::Error + Send + Sync>> {
            let filtered_events: Vec<Box<dyn DomainEvent + Send>> = self
                .events
                .iter()
                .filter(|event| event.event_type() == event_type)
                .map(|event| {
                    let simple_event = SimpleDomainEvent {
                        event_type: event.event_type().to_string(),
                        event_id: event.event_id(),
                        timestamp: event.timestamp(),
                        aggregate_id: event.aggregate_id().to_string(),
                        version: event.version(),
                        data: event.data().clone(),
                    };
                    Box::new(simple_event) as Box<dyn DomainEvent + Send>
                })
                .collect();
            Ok(filtered_events)
        }
    }

    // Mock observer for testing
    struct MockEventObserver {
        name: String,
        event_types: Vec<String>,
        handled_events: Arc<Mutex<Vec<String>>>,
        should_fail: bool,
    }

    impl MockEventObserver {
        fn new(name: &str, event_types: Vec<String>) -> Self {
            Self {
                name: name.to_string(),
                event_types,
                handled_events: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
            }
        }

        fn with_failure(mut self, should_fail: bool) -> Self {
            self.should_fail = should_fail;
            self
        }

        fn get_handled_events(&self) -> Vec<String> {
            self.handled_events.lock().unwrap().clone()
        }
    }

    impl EventObserver for MockEventObserver {
        fn handle_event(&self, event: &dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if self.should_fail {
                return Err("Mock observer failed".into());
            }

            let mut events = self.handled_events.lock().unwrap();
            events.push(format!("{}:{}", event.event_type(), event.aggregate_id()));
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_interested_in(&self, event_type: &str) -> bool {
            self.event_types.is_empty() || self.event_types.contains(&event_type.to_string())
        }
    }

    // Tests for DomainEvent trait
    #[test]
    fn test_domain_event_trait_methods() {
        let data = serde_yaml::to_value("test data").unwrap();
        let event = SimpleDomainEvent::new("test_event", "agg_001", data.clone());

        assert_eq!(event.event_type(), "test_event");
        assert_eq!(event.aggregate_id(), "agg_001");
        assert_eq!(event.version(), 1);
        assert_eq!(event.data(), data);
        assert!(!event.event_id().to_string().is_empty());
        assert!(event.timestamp() <= chrono::Utc::now());
    }

    #[test]
    fn test_simple_domain_event_new() {
        let data = serde_yaml::to_value("event data").unwrap();
        let event = SimpleDomainEvent::new("user_created", "user_123", data.clone());

        assert_eq!(event.event_type, "user_created");
        assert_eq!(event.aggregate_id, "user_123");
        assert_eq!(event.data, data);
        assert_eq!(event.version, 1);
    }

    #[test]
    fn test_simple_domain_event_with_version() {
        let data = serde_yaml::to_value("versioned data").unwrap();
        let event = SimpleDomainEvent::new("versioned_event", "agg_002", data).with_version(5);

        assert_eq!(event.version, 5);
    }

    #[test]
    fn test_simple_domain_event_with_timestamp() {
        let data = serde_yaml::to_value("timestamped data").unwrap();
        let timestamp = chrono::Utc::now();
        let event = SimpleDomainEvent::new("timestamped_event", "agg_003", data).with_timestamp(timestamp);

        assert_eq!(event.timestamp, timestamp);
    }

    // Tests for EventObserver trait
    #[test]
    fn test_event_observer_trait_methods() {
        let observer = MockEventObserver::new("test_observer", vec!["test_event".to_string()]);

        assert_eq!(observer.name(), "test_observer");
        assert!(observer.is_interested_in("test_event"));
        assert!(!observer.is_interested_in("other_event"));
    }

    #[test]
    fn test_event_observer_universal() {
        let observer = MockEventObserver::new("universal_observer", vec![]);

        assert!(observer.is_interested_in("any_event"));
        assert!(observer.is_interested_in("another_event"));
    }

    // Tests for SimpleEventObserver
    #[test]
    fn test_simple_event_observer_new() {
        let observer = SimpleEventObserver::new(
            "simple_observer",
            vec!["user_created".to_string(), "user_updated".to_string()],
            |_event| Ok(()),
        );

        assert_eq!(observer.name(), "simple_observer");
        assert!(observer.is_interested_in("user_created"));
        assert!(observer.is_interested_in("user_updated"));
        assert!(!observer.is_interested_in("user_deleted"));
    }

    #[test]
    fn test_simple_event_observer_new_universal() {
        let observer = SimpleEventObserver::new_universal("universal_observer", |_event| Ok(()));

        assert_eq!(observer.name(), "universal_observer");
        assert!(observer.is_interested_in("any_event"));
        assert!(observer.is_interested_in("another_event"));
    }

    #[test]
    fn test_simple_event_observer_handle_event() {
        let handled_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&handled_events);

        let observer = SimpleEventObserver::new("handler_observer", vec!["test_event".to_string()], move |event| {
            let mut events = events_clone.lock().unwrap();
            events.push(event.event_type().to_string());
            Ok(())
        });

        let data = serde_yaml::to_value("test").unwrap();
        let event = SimpleDomainEvent::new("test_event", "agg_004", data);

        let result = observer.handle_event(&event);
        assert!(result.is_ok());

        let events = handled_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "test_event");
    }

    // Tests for EventBus
    #[test]
    fn test_event_bus_new() {
        let event_bus = EventBus::new();
        assert_eq!(event_bus.observer_count("any_event"), 0);
    }

    #[test]
    fn test_event_bus_default() {
        let event_bus = EventBus::default();
        assert_eq!(event_bus.observer_count("any_event"), 0);
    }

    #[test]
    fn test_event_bus_clone() {
        let event_bus = EventBus::new();
        let cloned_bus = event_bus.clone();

        assert_eq!(event_bus.observer_count("any_event"), 0);
        assert_eq!(cloned_bus.observer_count("any_event"), 0);
    }

    #[test]
    fn test_event_bus_subscribe() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new("test_observer", vec!["test_event".to_string()]));

        event_bus.subscribe("test_event", observer);
        assert_eq!(event_bus.observer_count("test_event"), 1);
        assert_eq!(event_bus.observer_count("other_event"), 0);
    }

    #[test]
    fn test_event_bus_subscribe_multiple_observers() {
        let event_bus = EventBus::new();
        let observer1 = Arc::new(MockEventObserver::new("observer1", vec!["test_event".to_string()]));
        let observer2 = Arc::new(MockEventObserver::new("observer2", vec!["test_event".to_string()]));

        event_bus.subscribe("test_event", observer1);
        event_bus.subscribe("test_event", observer2);

        assert_eq!(event_bus.observer_count("test_event"), 2);
    }

    #[test]
    fn test_event_bus_subscribe_different_event_types() {
        let event_bus = EventBus::new();
        let observer1 = Arc::new(MockEventObserver::new("observer1", vec!["event_type_1".to_string()]));
        let observer2 = Arc::new(MockEventObserver::new("observer2", vec!["event_type_2".to_string()]));

        event_bus.subscribe("event_type_1", observer1);
        event_bus.subscribe("event_type_2", observer2);

        assert_eq!(event_bus.observer_count("event_type_1"), 1);
        assert_eq!(event_bus.observer_count("event_type_2"), 1);
    }

    #[test]
    fn test_event_bus_unsubscribe() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new("test_observer", vec!["test_event".to_string()]));

        event_bus.subscribe("test_event", observer.clone());
        assert_eq!(event_bus.observer_count("test_event"), 1);

        event_bus.unsubscribe("test_event", "test_observer");
        assert_eq!(event_bus.observer_count("test_event"), 0);
    }

    #[test]
    fn test_event_bus_unsubscribe_nonexistent() {
        let event_bus = EventBus::new();
        // Should not panic
        event_bus.unsubscribe("nonexistent_event", "nonexistent_observer");
    }

    #[test]
    fn test_event_bus_publish() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new("test_observer", vec!["test_event".to_string()]));

        event_bus.subscribe("test_event", observer);

        let data = serde_yaml::to_value("publish test").unwrap();
        let event = SimpleDomainEvent::new("test_event", "agg_005", data);

        let result = event_bus.publish(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_bus_publish_no_observers() {
        let event_bus = EventBus::new();
        let data = serde_yaml::to_value("no observers").unwrap();
        let event = SimpleDomainEvent::new("test_event", "agg_006", data);

        let result = event_bus.publish(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_bus_publish_observer_failure() {
        let event_bus = EventBus::new();
        let observer =
            Arc::new(MockEventObserver::new("failing_observer", vec!["test_event".to_string()]).with_failure(true));

        event_bus.subscribe("test_event", observer);

        let data = serde_yaml::to_value("failing test").unwrap();
        let event = SimpleDomainEvent::new("test_event", "agg_007", data);

        // Should not panic, should handle the error gracefully
        let result = event_bus.publish(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_bus_publish_async() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new(
            "async_observer",
            vec!["async_event".to_string()],
        ));

        event_bus.subscribe("async_event", observer);

        let data = serde_yaml::to_value("async test").unwrap();
        let event = Box::new(SimpleDomainEvent::new("async_event", "agg_008", data));

        // Should not panic
        event_bus.publish_async(event);

        // Give some time for the async operation to complete
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_event_bus_observer_count() {
        let event_bus = EventBus::new();
        let observer1 = Arc::new(MockEventObserver::new("observer1", vec!["test_event".to_string()]));
        let observer2 = Arc::new(MockEventObserver::new("observer2", vec!["test_event".to_string()]));

        assert_eq!(event_bus.observer_count("test_event"), 0);

        event_bus.subscribe("test_event", observer1);
        assert_eq!(event_bus.observer_count("test_event"), 1);

        event_bus.subscribe("test_event", observer2);
        assert_eq!(event_bus.observer_count("test_event"), 2);
    }

    #[test]
    fn test_event_bus_event_types() {
        let event_bus = EventBus::new();
        let observer1 = Arc::new(MockEventObserver::new("observer1", vec!["event_type_1".to_string()]));
        let observer2 = Arc::new(MockEventObserver::new("observer2", vec!["event_type_2".to_string()]));

        event_bus.subscribe("event_type_1", observer1);
        event_bus.subscribe("event_type_2", observer2);

        let event_types = event_bus.event_types();
        assert_eq!(event_types.len(), 2);
        assert!(event_types.contains(&"event_type_1".to_string()));
        assert!(event_types.contains(&"event_type_2".to_string()));
    }

    // Tests for EventReplayer
    #[test]
    fn test_event_replayer_new() {
        let event_store = Box::new(MockEventStore::new());
        let event_bus = EventBus::new();
        let _replayer = EventReplayer::new(event_store, event_bus);

        // Just test that it can be created without errors
        assert!(true);
    }

    #[test]
    fn test_event_replayer_replay_aggregate_events() {
        let event_store = MockEventStore::new();
        let event_bus = EventBus::new();
        let replayer = EventReplayer::new(Box::new(event_store), event_bus);

        // Test that it can be called without errors
        let result = replayer.replay_aggregate_events("test_aggregate");
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_replayer_replay_events_by_type() {
        let event_store = MockEventStore::new();
        let event_bus = EventBus::new();
        let replayer = EventReplayer::new(Box::new(event_store), event_bus);

        // Test that it can be called without errors
        let result = replayer.replay_events_by_type("test_event_type");
        assert!(result.is_ok());
    }

    // Tests for complex scenarios
    #[test]
    fn test_event_bus_complex_scenario() {
        let event_bus = EventBus::new();

        // Subscribe multiple observers to different event types
        let observer1 = Arc::new(MockEventObserver::new("observer1", vec!["user_created".to_string()]));
        let observer2 = Arc::new(MockEventObserver::new("observer2", vec!["user_updated".to_string()]));
        let observer3 = Arc::new(MockEventObserver::new(
            "observer3",
            vec!["user_created".to_string(), "user_updated".to_string()],
        ));

        event_bus.subscribe("user_created", observer1);
        event_bus.subscribe("user_updated", observer2);
        event_bus.subscribe("user_created", observer3.clone());
        event_bus.subscribe("user_updated", observer3);

        // Verify observer counts
        assert_eq!(event_bus.observer_count("user_created"), 2);
        assert_eq!(event_bus.observer_count("user_updated"), 2);

        // Verify event types
        let event_types = event_bus.event_types();
        assert_eq!(event_types.len(), 2);
        assert!(event_types.contains(&"user_created".to_string()));
        assert!(event_types.contains(&"user_updated".to_string()));
    }

    #[test]
    fn test_event_bus_observer_interest_filtering() {
        let event_bus = EventBus::new();

        // Create observers with specific interests
        let user_observer = Arc::new(MockEventObserver::new(
            "user_observer",
            vec!["user_created".to_string()],
        ));
        let project_observer = Arc::new(MockEventObserver::new(
            "project_observer",
            vec!["project_created".to_string()],
        ));
        let universal_observer = Arc::new(MockEventObserver::new("universal_observer", vec![]));

        event_bus.subscribe("user_created", user_observer);
        event_bus.subscribe("project_created", project_observer);
        event_bus.subscribe("any_event", universal_observer);

        // Verify that observers are only interested in their specific events
        assert_eq!(event_bus.observer_count("user_created"), 1);
        assert_eq!(event_bus.observer_count("project_created"), 1);
        assert_eq!(event_bus.observer_count("any_event"), 1);
        assert_eq!(event_bus.observer_count("unrelated_event"), 0);
    }

    // Tests for edge cases
    #[test]
    fn test_event_bus_empty_event_type() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new("empty_observer", vec!["".to_string()]));

        event_bus.subscribe("", observer);
        assert_eq!(event_bus.observer_count(""), 1);
    }

    #[test]
    fn test_event_bus_special_characters_in_event_type() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new(
            "special_observer",
            vec!["event-with-dashes".to_string()],
        ));

        event_bus.subscribe("event-with-dashes", observer);
        assert_eq!(event_bus.observer_count("event-with-dashes"), 1);
    }

    #[test]
    fn test_event_bus_unicode_event_type() {
        let event_bus = EventBus::new();
        let observer = Arc::new(MockEventObserver::new(
            "unicode_observer",
            vec!["événement_événement".to_string()],
        ));

        event_bus.subscribe("événement_événement", observer);
        assert_eq!(event_bus.observer_count("événement_événement"), 1);
    }
}
