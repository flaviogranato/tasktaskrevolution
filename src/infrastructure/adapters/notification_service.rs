//! Notification service adapter implementation
//!
//! This module provides a concrete implementation of the NotificationServicePort
//! for sending notifications.

use crate::domain::ports::notification_service::{
    NotificationServicePort, DomainNotification, NotificationServiceStatus,
};
use crate::domain::shared::errors::{DomainError, DomainResult};

/// Standard notification service adapter
pub struct StandardNotificationServiceAdapter {
    enabled: bool,
}

impl StandardNotificationServiceAdapter {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Default for StandardNotificationServiceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationServicePort for StandardNotificationServiceAdapter {
    fn send_notification(&self, notification: DomainNotification) -> DomainResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // In a real implementation, this would send the notification
        // For now, we'll just log it
        println!(
            "Notification sent: {} to {} ({:?})",
            notification.title,
            notification.recipient.name,
            notification.notification_type
        );

        Ok(())
    }

    fn send_batch(&self, notifications: Vec<DomainNotification>) -> DomainResult<()> {
        if !self.enabled {
            return Ok(());
        }

        for notification in notifications {
            self.send_notification(notification)?;
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        self.enabled
    }

    fn status(&self) -> NotificationServiceStatus {
        if self.enabled {
            NotificationServiceStatus::Available
        } else {
            NotificationServiceStatus::Unavailable
        }
    }
}
