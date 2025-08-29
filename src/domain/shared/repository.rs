use crate::domain::shared::errors::DomainError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A generic repository trait for domain entities
pub trait Repository<T, ID> {
    /// Find an entity by ID
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, DomainError>;

    /// Find all entities
    fn find_all(&self) -> Result<Vec<T>, DomainError>;

    /// Save an entity
    fn save(&self, entity: T) -> Result<T, DomainError>;

    /// Update an existing entity
    fn update(&self, entity: T) -> Result<T, DomainError>;

    /// Delete an entity by ID
    fn delete(&self, id: &ID) -> Result<bool, DomainError>;

    /// Check if an entity exists
    fn exists(&self, id: &ID) -> Result<bool, DomainError>;

    /// Count all entities
    fn count(&self) -> Result<usize, DomainError>;
}

/// A repository that supports pagination
pub trait PaginatedRepository<T, ID>: Repository<T, ID> {
    /// Find entities with pagination
    fn find_with_pagination(&self, page: usize, size: usize) -> Result<PaginatedResult<T>, DomainError>;
}

/// A repository that supports searching
pub trait SearchableRepository<T, ID>: Repository<T, ID> {
    /// Search entities by criteria
    fn search(&self, criteria: &SearchCriteria) -> Result<Vec<T>, DomainError>;

    /// Search entities with pagination
    fn search_with_pagination(
        &self,
        criteria: &SearchCriteria,
        page: usize,
        size: usize,
    ) -> Result<PaginatedResult<T>, DomainError>;
}

/// A repository that supports transactions
pub trait TransactionalRepository<T, ID>: Repository<T, ID> {
    /// Begin a transaction
    fn begin_transaction(&self) -> Result<Box<dyn Transaction>, DomainError>;

    /// Execute a function within a transaction
    fn with_transaction<F, R>(&self, f: F) -> Result<R, DomainError>
    where
        F: FnOnce(&dyn Transaction) -> Result<R, DomainError>;
}

/// A transaction that can be committed or rolled back
pub trait Transaction {
    /// Commit the transaction
    fn commit(self: Box<Self>) -> Result<(), DomainError>;

    /// Rollback the transaction
    fn rollback(self: Box<Self>) -> Result<(), DomainError>;
}

/// Search criteria for repositories
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub filters: HashMap<String, String>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

impl SearchCriteria {
    /// Create new search criteria
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            sort_by: None,
            sort_order: SortOrder::Ascending,
        }
    }

    /// Add a filter
    pub fn with_filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.insert(key.into(), value.into());
        self
    }

    /// Set sort field
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    /// Set sort order
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Paginated result
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
        let total_pages = if total == 0 { 0 } else { total.div_ceil(size) };

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

/// A repository that caches results
pub trait CachedRepository<T, ID>: Repository<T, ID> {
    /// Get the cache key for an entity
    fn cache_key(&self, id: &ID) -> String;

    /// Invalidate cache for an entity
    fn invalidate_cache(&self, id: &ID) -> Result<(), DomainError>;

    /// Clear all cache
    fn clear_cache(&self) -> Result<(), DomainError>;
}

/// A simple in-memory repository implementation
pub struct InMemoryRepository<T, ID> {
    entities: Arc<Mutex<HashMap<ID, T>>>,
}

impl<T, ID> InMemoryRepository<T, ID>
where
    ID: Clone + std::hash::Hash + Eq,
    T: Clone,
{
    /// Create a new in-memory repository
    pub fn new() -> Self {
        Self {
            entities: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a reference to the entities map
    fn entities(&self) -> std::sync::MutexGuard<'_, HashMap<ID, T>> {
        self.entities.lock().unwrap()
    }
}

impl<T, ID> Repository<T, ID> for InMemoryRepository<T, ID>
where
    ID: Clone + std::hash::Hash + Eq,
    T: Clone,
{
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, DomainError> {
        let entities = self.entities();
        Ok(entities.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<T>, DomainError> {
        let entities = self.entities();
        Ok(entities.values().cloned().collect())
    }

    fn save(&self, entity: T) -> Result<T, DomainError> {
        // This is a simplified implementation - in a real scenario,
        // you'd need to extract the ID from the entity
        let _entities = self.entities();
        // For now, we'll just return the entity as-is
        Ok(entity)
    }

    fn update(&self, entity: T) -> Result<T, DomainError> {
        // Similar to save for this simple implementation
        Ok(entity)
    }

    fn delete(&self, id: &ID) -> Result<bool, DomainError> {
        let mut entities = self.entities();
        Ok(entities.remove(id).is_some())
    }

    fn exists(&self, id: &ID) -> Result<bool, DomainError> {
        let entities = self.entities();
        Ok(entities.contains_key(id))
    }

    fn count(&self) -> Result<usize, DomainError> {
        let entities = self.entities();
        Ok(entities.len())
    }
}

impl<T, ID> Default for InMemoryRepository<T, ID>
where
    ID: Clone + std::hash::Hash + Eq,
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A repository decorator that adds logging
pub struct LoggingRepositoryDecorator<T, ID, R> {
    repository: R,
    _phantom: std::marker::PhantomData<(T, ID)>,
}

impl<T, ID, R> LoggingRepositoryDecorator<T, ID, R>
where
    R: Repository<T, ID>,
    ID: std::fmt::Debug,
{
    /// Create a new logging repository decorator
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, ID, R> Repository<T, ID> for LoggingRepositoryDecorator<T, ID, R>
where
    R: Repository<T, ID>,
    ID: std::fmt::Debug,
{
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, DomainError> {
        println!("Repository: Finding entity by ID: {:?}", id);
        let result = self.repository.find_by_id(id);
        match &result {
            Ok(Some(_)) => println!("Repository: Entity found"),
            Ok(None) => println!("Repository: Entity not found"),
            Err(e) => println!("Repository: Error finding entity: {}", e),
        }
        result
    }

    fn find_all(&self) -> Result<Vec<T>, DomainError> {
        println!("Repository: Finding all entities");
        let result = self.repository.find_all();
        match &result {
            Ok(entities) => println!("Repository: Found {} entities", entities.len()),
            Err(e) => println!("Repository: Error finding all entities: {}", e),
        }
        result
    }

    fn save(&self, entity: T) -> Result<T, DomainError> {
        println!("Repository: Saving entity");
        let result = self.repository.save(entity);
        match &result {
            Ok(_) => println!("Repository: Entity saved successfully"),
            Err(e) => println!("Repository: Error saving entity: {}", e),
        }
        result
    }

    fn update(&self, entity: T) -> Result<T, DomainError> {
        println!("Repository: Updating entity");
        let result = self.repository.update(entity);
        match &result {
            Ok(_) => println!("Repository: Entity updated successfully"),
            Err(e) => println!("Repository: Error updating entity: {}", e),
        }
        result
    }

    fn delete(&self, id: &ID) -> Result<bool, DomainError> {
        println!("Repository: Deleting entity with ID: {:?}", id);
        let result = self.repository.delete(id);
        match &result {
            Ok(deleted) => println!("Repository: Entity {}deleted", if *deleted { "" } else { "not " }),
            Err(e) => println!("Repository: Error deleting entity: {}", e),
        }
        result
    }

    fn exists(&self, id: &ID) -> Result<bool, DomainError> {
        println!("Repository: Checking if entity exists with ID: {:?}", id);
        let result = self.repository.exists(id);
        match &result {
            Ok(exists) => println!("Repository: Entity {}exists", if *exists { "" } else { "does not " }),
            Err(e) => println!("Repository: Error checking entity existence: {}", e),
        }
        result
    }

    fn count(&self) -> Result<usize, DomainError> {
        println!("Repository: Counting entities");
        let result = self.repository.count();
        match &result {
            Ok(count) => println!("Repository: Found {} entities", count),
            Err(e) => println!("Repository: Error counting entities: {}", e),
        }
        result
    }
}
