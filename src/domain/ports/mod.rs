//! Domain ports for dependency inversion
//!
//! This module defines the interfaces (ports) that the domain layer
//! requires from external layers, following the Ports and Adapters pattern.
//! These ports define contracts that infrastructure adapters must implement.

pub mod event_publisher;
pub mod file_system;
pub mod id_generator;
pub mod notification_service;
pub mod repository;
pub mod time_service;
pub mod validation_service;

pub use event_publisher::*;
pub use file_system::*;
pub use id_generator::*;
pub use notification_service::*;
pub use repository::*;
pub use time_service::*;
pub use validation_service::*;
