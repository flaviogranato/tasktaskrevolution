//! Event publisher port for domain events
//!
//! This module defines the event publishing interface that the domain layer
//! requires from the infrastructure layer.

use crate::domain::shared::errors::DomainResult;
use std::sync::Arc;

/// Domain event that can be published
pub trait DomainEvent: Send + Sync {
    /// Get the event type
    fn event_type(&self) -> &str;

    /// Get the event ID
    fn event_id(&self) -> String;

    /// Get the timestamp when the event occurred
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the aggregate ID that generated this event
    fn aggregate_id(&self) -> &str;

    /// Get the event version
    fn version(&self) -> u64;

    /// Get the event data as JSON
    fn data(&self) -> serde_json::Value;
}

/// Event publisher port for publishing domain events
pub trait EventPublisherPort: Send + Sync {
    /// Publish a domain event
    fn publish(&self, event: Box<dyn DomainEvent>) -> DomainResult<()>;

    /// Publish multiple events in a batch
    fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> DomainResult<()>;

    /// Check if the publisher is available
    fn is_available(&self) -> bool;
}

/// Event subscriber for handling domain events
pub trait EventSubscriber: Send + Sync {
    /// Handle a domain event
    fn handle_event(&self, event: &dyn DomainEvent) -> DomainResult<()>;

    /// Get the subscriber name
    fn name(&self) -> &str;

    /// Check if the subscriber is interested in a specific event type
    fn is_interested_in(&self, event_type: &str) -> bool;
}

/// Event bus for managing event publishing and subscription
pub trait EventBusPort: Send + Sync {
    /// Subscribe to events
    fn subscribe(&self, subscriber: Arc<dyn EventSubscriber>) -> DomainResult<()>;

    /// Unsubscribe from events
    fn unsubscribe(&self, subscriber_name: &str) -> DomainResult<()>;

    /// Publish an event
    fn publish(&self, event: Box<dyn DomainEvent>) -> DomainResult<()>;

    /// Get the number of subscribers
    fn subscriber_count(&self) -> usize;
}
