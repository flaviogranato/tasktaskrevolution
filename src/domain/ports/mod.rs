//! Domain ports for dependency inversion
//!
//! This module defines the interfaces (ports) that the domain layer
//! requires from external layers, following the Ports and Adapters pattern.
//! These ports define contracts that infrastructure adapters must implement.

pub mod repository;
pub mod event_publisher;
pub mod notification_service;
pub mod file_system;
pub mod time_service;
pub mod id_generator;
pub mod validation_service;

pub use repository::*;
pub use event_publisher::*;
pub use notification_service::*;
pub use file_system::*;
pub use time_service::*;
pub use id_generator::*;
pub use validation_service::*;
