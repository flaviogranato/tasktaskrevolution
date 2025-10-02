//! Adapter tests for repository implementations
//! 
//! This module contains unit tests for the repository adapters (implementations)
//! that verify the correct behavior of the infrastructure layer components.
//! 
//! These tests use fixtures to create isolated test environments and verify
//! that the repository implementations correctly fulfill their contracts
//! as defined by the domain traits.

pub mod company_repository_test;
pub mod project_repository_test;
pub mod resource_repository_test;
pub mod task_repository_test;
