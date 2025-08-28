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
        observers
            .entry(event_type.into())
            .or_insert_with(Vec::new)
            .push(observer);
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
                if observer.is_interested_in(event_type) {
                    if let Err(e) = observer.handle_event(event) {
                        eprintln!("Observer {} failed to handle event: {}", observer.name(), e);
                    }
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
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        data: serde_yaml::Value,
    ) -> Self {
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
    pub fn new<F>(
        name: impl Into<String>,
        event_types: Vec<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(&dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            event_types,
            handler: Box::new(handler),
        }
    }
    
    /// Create an observer that handles all event types
    pub fn new_universal<F>(
        name: impl Into<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(&dyn DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
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
    fn get_events(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent + Send>>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Retrieve events of a specific type
    fn get_events_by_type(&self, event_type: &str) -> Result<Vec<Box<dyn DomainEvent + Send>>, Box<dyn std::error::Error + Send + Sync>>;
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
        Self {
            event_store,
            event_bus,
        }
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
