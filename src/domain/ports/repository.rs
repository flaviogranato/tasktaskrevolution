//! Repository ports for domain entities
//!
//! This module defines the repository interfaces that the domain layer
//! requires from the infrastructure layer, following the Repository pattern.

use crate::domain::shared::errors::{DomainError, DomainResult};
use std::collections::HashMap;

/// Generic repository port for domain entities
pub trait RepositoryPort<T, ID> {
    /// Find an entity by ID
    fn find_by_id(&self, id: &ID) -> DomainResult<Option<T>>;

    /// Find all entities
    fn find_all(&self) -> DomainResult<Vec<T>>;

    /// Save an entity
    fn save(&self, entity: T) -> DomainResult<T>;

    /// Update an existing entity
    fn update(&self, entity: T) -> DomainResult<T>;

    /// Delete an entity by ID
    fn delete(&self, id: &ID) -> DomainResult<bool>;

    /// Check if an entity exists
    fn exists(&self, id: &ID) -> DomainResult<bool>;

    /// Count all entities
    fn count(&self) -> DomainResult<usize>;
}

/// Repository port that supports pagination
pub trait PaginatedRepositoryPort<T, ID>: RepositoryPort<T, ID> {
    /// Find entities with pagination
    fn find_with_pagination(&self, page: usize, size: usize) -> DomainResult<PaginatedResult<T>>;
}

/// Repository port that supports searching
pub trait SearchableRepositoryPort<T, ID>: RepositoryPort<T, ID> {
    /// Search entities by criteria
    fn search(&self, criteria: &SearchCriteria) -> DomainResult<Vec<T>>;

    /// Search entities with pagination
    fn search_with_pagination(
        &self,
        criteria: &SearchCriteria,
        page: usize,
        size: usize,
    ) -> DomainResult<PaginatedResult<T>>;
}

/// Repository port that supports transactions
pub trait TransactionalRepositoryPort<T, ID>: RepositoryPort<T, ID> {
    /// Begin a transaction
    fn begin_transaction(&self) -> DomainResult<Box<dyn TransactionPort>>;

    /// Execute a function within a transaction
    fn with_transaction<F, R>(&self, f: F) -> DomainResult<R>
    where
        F: FnOnce(&dyn TransactionPort) -> DomainResult<R>;
}

/// Repository port that supports caching
pub trait CachedRepositoryPort<T, ID>: RepositoryPort<T, ID> {
    /// Get the cache key for an entity
    fn cache_key(&self, id: &ID) -> String;

    /// Invalidate cache for an entity
    fn invalidate_cache(&self, id: &ID) -> DomainResult<()>;

    /// Clear all cache
    fn clear_cache(&self) -> DomainResult<()>;
}

/// Transaction port for database operations
pub trait TransactionPort {
    /// Commit the transaction
    fn commit(&self) -> DomainResult<()>;

    /// Rollback the transaction
    fn rollback(&self) -> DomainResult<()>;

    /// Check if the transaction is active
    fn is_active(&self) -> bool;
}

/// Search criteria for repository queries
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub filters: HashMap<String, String>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            filters: HashMap::new(),
            sort_by: None,
            sort_order: SortOrder::Ascending,
        }
    }
}

/// Sort order for search results
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Paginated result for repository queries
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub size: usize,
    pub total: usize,
    pub total_pages: usize,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(items: Vec<T>, page: usize, size: usize, total: usize) -> Self {
        let total_pages = if total == 0 || size == 0 {
            0
        } else {
            total.div_ceil(size)
        };

        Self {
            items,
            page,
            size,
            total,
            total_pages,
        }
    }

    /// Check if there's a next page
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there's a previous page
    pub fn has_previous(&self) -> bool {
        self.page > 1
    }

    /// Get the next page number
    pub fn next_page(&self) -> Option<usize> {
        if self.has_next() { Some(self.page + 1) } else { None }
    }

    /// Get the previous page number
    pub fn previous_page(&self) -> Option<usize> {
        if self.has_previous() { Some(self.page - 1) } else { None }
    }
}
