//! Event publisher adapter implementation
//!
//! This module provides a concrete implementation of the EventPublisherPort
//! for publishing domain events.

use crate::domain::ports::event_publisher::{
    EventPublisherPort, EventBusPort, DomainEvent, EventSubscriber,
};
use crate::domain::shared::errors::DomainResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Standard event publisher adapter
pub struct StandardEventPublisherAdapter {
    subscribers: Arc<Mutex<HashMap<String, Arc<dyn EventSubscriber>>>>,
}

impl StandardEventPublisherAdapter {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for StandardEventPublisherAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisherPort for StandardEventPublisherAdapter {
    fn publish(&self, event: Box<dyn DomainEvent>) -> DomainResult<()> {
        let subscribers = self.subscribers.lock().unwrap();
        
        for (_, subscriber) in subscribers.iter() {
            if subscriber.is_interested_in(event.event_type())
                && let Err(e) = subscriber.handle_event(event.as_ref()) {
                eprintln!("Error handling event: {}", e);
            }
        }

        Ok(())
    }

    fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> DomainResult<()> {
        for event in events {
            EventPublisherPort::publish(self, event)?;
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }
}

impl EventBusPort for StandardEventPublisherAdapter {
    fn subscribe(&self, subscriber: Arc<dyn EventSubscriber>) -> DomainResult<()> {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.insert(subscriber.name().to_string(), subscriber);
        Ok(())
    }

    fn unsubscribe(&self, subscriber_name: &str) -> DomainResult<()> {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.remove(subscriber_name);
        Ok(())
    }

    fn publish(&self, event: Box<dyn DomainEvent>) -> DomainResult<()> {
        EventPublisherPort::publish(self, event)
    }

    fn subscriber_count(&self) -> usize {
        let subscribers = self.subscribers.lock().unwrap();
        subscribers.len()
    }
}
