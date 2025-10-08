//! Notification service port for domain notifications
//!
//! This module defines the notification interface that the domain layer
//! requires from the infrastructure layer.

use crate::domain::shared::errors::{DomainError, DomainResult};

/// Notification priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Notification types
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Email,
    Sms,
    Push,
    InApp,
    Webhook,
}

/// Notification recipient
#[derive(Debug, Clone)]
pub struct NotificationRecipient {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub preferences: NotificationPreferences,
}

/// Notification preferences for a recipient
#[derive(Debug, Clone)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub in_app_enabled: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            email_enabled: true,
            sms_enabled: false,
            push_enabled: true,
            in_app_enabled: true,
        }
    }
}

/// Domain notification
#[derive(Debug, Clone)]
pub struct DomainNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub recipient: NotificationRecipient,
    pub metadata: std::collections::HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Notification service port for sending notifications
pub trait NotificationServicePort: Send + Sync {
    /// Send a notification
    fn send_notification(&self, notification: DomainNotification) -> DomainResult<()>;

    /// Send multiple notifications in a batch
    fn send_batch(&self, notifications: Vec<DomainNotification>) -> DomainResult<()>;

    /// Check if the service is available
    fn is_available(&self) -> bool;

    /// Get the service status
    fn status(&self) -> NotificationServiceStatus;
}

/// Notification service status
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationServiceStatus {
    Available,
    Unavailable,
    Maintenance,
    Error(String),
}
