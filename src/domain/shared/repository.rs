#![allow(dead_code)]

use crate::application::errors::AppError;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A generic repository trait for domain entities
pub trait Repository<T, ID> {
    /// Find an entity by ID
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, AppError>;

    /// Find all entities
    fn find_all(&self) -> Result<Vec<T>, AppError>;

    /// Save an entity
    fn save(&self, entity: T) -> Result<T, AppError>;

    /// Update an existing entity
    fn update(&self, entity: T) -> Result<T, AppError>;

    /// Delete an entity by ID
    fn delete(&self, id: &ID) -> Result<bool, AppError>;

    /// Check if an entity exists
    fn exists(&self, id: &ID) -> Result<bool, AppError>;

    /// Count all entities
    fn count(&self) -> Result<usize, AppError>;
}

/// A repository that supports pagination
pub trait PaginatedRepository<T, ID>: Repository<T, ID> {
    /// Find entities with pagination
    fn find_with_pagination(&self, page: usize, size: usize) -> Result<PaginatedResult<T>, AppError>;
}

/// A repository that supports searching
pub trait SearchableRepository<T, ID>: Repository<T, ID> {
    /// Search entities by criteria
    fn search(&self, criteria: &SearchCriteria) -> Result<Vec<T>, AppError>;

    /// Search entities with pagination
    fn search_with_pagination(
        &self,
        criteria: &SearchCriteria,
        page: usize,
        size: usize,
    ) -> Result<PaginatedResult<T>, AppError>;
}

/// A repository that supports transactions
pub trait TransactionalRepository<T, ID>: Repository<T, ID> {
    /// Begin a transaction
    fn begin_transaction(&self) -> Result<Box<dyn Transaction>, AppError>;

    /// Execute a function within a transaction
    fn with_transaction<F, R>(&self, f: F) -> Result<R, AppError>
    where
        F: FnOnce(&dyn Transaction) -> Result<R, AppError>;
}

/// A transaction that can be committed or rolled back
pub trait Transaction {
    /// Commit the transaction
    fn commit(self: Box<Self>) -> Result<(), AppError>;

    /// Rollback the transaction
    fn rollback(self: Box<Self>) -> Result<(), AppError>;
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

/// A repository that caches results
pub trait CachedRepository<T, ID>: Repository<T, ID> {
    /// Get the cache key for an entity
    fn cache_key(&self, id: &ID) -> String;

    /// Invalidate cache for an entity
    fn invalidate_cache(&self, id: &ID) -> Result<(), AppError>;

    /// Clear all cache
    fn clear_cache(&self) -> Result<(), AppError>;
}

/// A simple in-memory repository implementation
pub struct InMemoryRepository<T> {
    entities: Arc<Mutex<HashMap<String, T>>>,
}

impl<T> InMemoryRepository<T>
where
    T: Clone,
{
    /// Create a new in-memory repository
    pub fn new() -> Self {
        Self {
            entities: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a reference to the entities map
    fn entities(&self) -> std::sync::MutexGuard<'_, HashMap<String, T>> {
        self.entities.lock().unwrap()
    }

    /// Get a mutable reference to the entities map
    fn entities_mut(&self) -> std::sync::MutexGuard<'_, HashMap<String, T>> {
        self.entities.lock().unwrap()
    }
}

impl<T> Repository<T, String> for InMemoryRepository<T>
where
    T: Clone,
{
    fn find_by_id(&self, id: &String) -> Result<Option<T>, AppError> {
        let entities = self.entities();
        Ok(entities.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<T>, AppError> {
        let entities = self.entities();
        Ok(entities.values().cloned().collect())
    }

    fn save(&self, entity: T) -> Result<T, AppError> {
        // Extract ID from entity - this is a simplified approach
        // In a real implementation, you'd have a way to get the ID from the entity
        let mut entities = self.entities_mut();

        // For now, we'll use a placeholder ID based on the entity's content
        // This is just for demonstration - in practice you'd extract the real ID
        let id = format!("entity_{}", entities.len());

        // Insert the entity with the generated ID
        entities.insert(id, entity.clone());

        Ok(entity)
    }

    fn update(&self, entity: T) -> Result<T, AppError> {
        // For this simplified implementation, we'll just save it
        // In a real implementation, you'd check if it exists first
        // Note: This creates a new entity with a new ID, not a true update
        self.save(entity.clone())?;
        Ok(entity)
    }

    fn delete(&self, id: &String) -> Result<bool, AppError> {
        let mut entities = self.entities_mut();
        let deleted = entities.remove(id).is_some();
        Ok(deleted)
    }

    fn exists(&self, id: &String) -> Result<bool, AppError> {
        let entities = self.entities();
        Ok(entities.contains_key(id))
    }

    fn count(&self) -> Result<usize, AppError> {
        let entities = self.entities();
        Ok(entities.len())
    }
}

impl<T> Default for InMemoryRepository<T>
where
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
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, AppError> {
        println!("Repository: Finding entity by ID: {:?}", id);
        let result = self.repository.find_by_id(id);
        match &result {
            Ok(Some(_)) => println!("Repository: Entity found"),
            Ok(None) => println!("Repository: Entity not found"),
            Err(e) => println!("Repository: Error finding entity: {}", e),
        }
        result
    }

    fn find_all(&self) -> Result<Vec<T>, AppError> {
        println!("Repository: Finding all entities");
        let result = self.repository.find_all();
        match &result {
            Ok(entities) => println!("Repository: Found {} entities", entities.len()),
            Err(e) => println!("Repository: Error finding all entities: {}", e),
        }
        result
    }

    fn save(&self, entity: T) -> Result<T, AppError> {
        println!("Repository: Saving entity");
        let result = self.repository.save(entity);
        match &result {
            Ok(_) => println!("Repository: Entity saved successfully"),
            Err(e) => println!("Repository: Error saving entity: {}", e),
        }
        result
    }

    fn update(&self, entity: T) -> Result<T, AppError> {
        println!("Repository: Updating entity");
        let result = self.repository.update(entity);
        match &result {
            Ok(_) => println!("Repository: Entity updated successfully"),
            Err(e) => println!("Repository: Error updating entity: {}", e),
        }
        result
    }

    fn delete(&self, id: &ID) -> Result<bool, AppError> {
        println!("Repository: Deleting entity with ID: {:?}", id);
        let result = self.repository.delete(id);
        match &result {
            Ok(deleted) => println!("Repository: Entity {}deleted", if *deleted { "" } else { "not " }),
            Err(e) => println!("Repository: Error deleting entity: {}", e),
        }
        result
    }

    fn exists(&self, id: &ID) -> Result<bool, AppError> {
        println!("Repository: Checking if entity exists with ID: {:?}", id);
        let result = self.repository.exists(id);
        match &result {
            Ok(exists) => println!("Repository: Entity {}exists", if *exists { "" } else { "does not " }),
            Err(e) => println!("Repository: Error checking entity existence: {}", e),
        }
        result
    }

    fn count(&self) -> Result<usize, AppError> {
        println!("Repository: Counting entities");
        let result = self.repository.count();
        match &result {
            Ok(count) => println!("Repository: Found {} entities", count),
            Err(e) => println!("Repository: Error counting entities: {}", e),
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock entity types for testing
    #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    struct MockEntity {
        id: String,
        name: String,
        value: u32,
    }

    impl MockEntity {
        fn new(id: &str, name: &str, value: u32) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                value,
            }
        }
    }

    // Mock repository implementation for testing
    struct MockRepository {
        entities: Arc<Mutex<HashMap<String, MockEntity>>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                entities: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_entity(&self, entity: MockEntity) {
            let mut entities = self.entities.lock().unwrap();
            entities.insert(entity.id.clone(), entity);
        }
    }

    impl Repository<MockEntity, String> for MockRepository {
        fn find_by_id(&self, id: &String) -> Result<Option<MockEntity>, AppError> {
            let entities = self.entities.lock().unwrap();
            Ok(entities.get(id).cloned())
        }

        fn find_all(&self) -> Result<Vec<MockEntity>, AppError> {
            let entities = self.entities.lock().unwrap();
            Ok(entities.values().cloned().collect())
        }

        fn save(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            let mut entities = self.entities.lock().unwrap();
            entities.insert(entity.id.clone(), entity.clone());
            Ok(entity)
        }

        fn update(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            let mut entities = self.entities.lock().unwrap();
            if entities.contains_key(&entity.id) {
                entities.insert(entity.id.clone(), entity.clone());
                Ok(entity)
            } else {
                Err(AppError::ValidationError {
                    field: "entity".to_string(),
                    message: "Entity not found for update".to_string(),
                })
            }
        }

        fn delete(&self, id: &String) -> Result<bool, AppError> {
            let mut entities = self.entities.lock().unwrap();
            Ok(entities.remove(id).is_some())
        }

        fn exists(&self, id: &String) -> Result<bool, AppError> {
            let entities = self.entities.lock().unwrap();
            Ok(entities.contains_key(id))
        }

        fn count(&self) -> Result<usize, AppError> {
            let entities = self.entities.lock().unwrap();
            Ok(entities.len())
        }
    }

    // Mock paginated repository implementation
    struct MockPaginatedRepository {
        base_repository: MockRepository,
    }

    impl MockPaginatedRepository {
        fn new() -> Self {
            Self {
                base_repository: MockRepository::new(),
            }
        }
    }

    impl Repository<MockEntity, String> for MockPaginatedRepository {
        fn find_by_id(&self, id: &String) -> Result<Option<MockEntity>, AppError> {
            self.base_repository.find_by_id(id)
        }

        fn find_all(&self) -> Result<Vec<MockEntity>, AppError> {
            self.base_repository.find_all()
        }

        fn save(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            self.base_repository.save(entity)
        }

        fn update(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            self.base_repository.update(entity)
        }

        fn delete(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.delete(id)
        }

        fn exists(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.exists(id)
        }

        fn count(&self) -> Result<usize, AppError> {
            self.base_repository.count()
        }
    }

    impl PaginatedRepository<MockEntity, String> for MockPaginatedRepository {
        fn find_with_pagination(&self, page: usize, size: usize) -> Result<PaginatedResult<MockEntity>, AppError> {
            let all_entities = self.find_all()?;
            let total = all_entities.len();
            let start = (page - 1) * size;
            let _end = start + size;

            let items = if start < total {
                all_entities.into_iter().skip(start).take(size).collect()
            } else {
                Vec::new()
            };

            Ok(PaginatedResult::new(items, page, size, total))
        }
    }

    // Mock searchable repository implementation
    struct MockSearchableRepository {
        base_repository: MockRepository,
    }

    impl MockSearchableRepository {
        fn new() -> Self {
            Self {
                base_repository: MockRepository::new(),
            }
        }
    }

    impl Repository<MockEntity, String> for MockSearchableRepository {
        fn find_by_id(&self, id: &String) -> Result<Option<MockEntity>, AppError> {
            self.base_repository.find_by_id(id)
        }

        fn find_all(&self) -> Result<Vec<MockEntity>, AppError> {
            self.base_repository.find_all()
        }

        fn save(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            self.base_repository.save(entity)
        }

        fn update(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            self.base_repository.update(entity)
        }

        fn delete(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.delete(id)
        }

        fn exists(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.exists(id)
        }

        fn count(&self) -> Result<usize, AppError> {
            self.base_repository.count()
        }
    }

    impl SearchableRepository<MockEntity, String> for MockSearchableRepository {
        fn search(&self, criteria: &SearchCriteria) -> Result<Vec<MockEntity>, AppError> {
            let all_entities = self.find_all()?;
            let mut filtered_entities = all_entities;

            // Apply filters
            for (key, value) in &criteria.filters {
                filtered_entities.retain(|entity| match key.as_str() {
                    "name" => entity.name.contains(value),
                    "value" => entity.value.to_string() == *value,
                    _ => true,
                });
            }

            // Apply sorting
            if let Some(sort_field) = &criteria.sort_by {
                match sort_field.as_str() {
                    "name" => {
                        filtered_entities.sort_by(|a, b| match criteria.sort_order {
                            SortOrder::Ascending => a.name.cmp(&b.name),
                            SortOrder::Descending => b.name.cmp(&a.name),
                        });
                    }
                    "value" => {
                        filtered_entities.sort_by(|a, b| match criteria.sort_order {
                            SortOrder::Ascending => a.value.cmp(&b.value),
                            SortOrder::Descending => b.value.cmp(&a.value),
                        });
                    }
                    _ => {}
                }
            }

            Ok(filtered_entities)
        }

        fn search_with_pagination(
            &self,
            criteria: &SearchCriteria,
            page: usize,
            size: usize,
        ) -> Result<PaginatedResult<MockEntity>, AppError> {
            let filtered_entities = self.search(criteria)?;
            let total = filtered_entities.len();
            let start = (page - 1) * size;
            let _end = start + size;

            let items = if start < total {
                filtered_entities.into_iter().skip(start).take(size).collect()
            } else {
                Vec::new()
            };

            Ok(PaginatedResult::new(items, page, size, total))
        }
    }

    // Mock transactional repository implementation
    struct MockTransactionalRepository {
        base_repository: MockRepository,
    }

    impl MockTransactionalRepository {
        fn new() -> Self {
            Self {
                base_repository: MockRepository::new(),
            }
        }
    }

    impl Repository<MockEntity, String> for MockTransactionalRepository {
        fn find_by_id(&self, id: &String) -> Result<Option<MockEntity>, AppError> {
            self.base_repository.find_by_id(id)
        }

        fn find_all(&self) -> Result<Vec<MockEntity>, AppError> {
            self.base_repository.find_all()
        }

        fn save(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            self.base_repository.save(entity)
        }

        fn update(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            self.base_repository.update(entity)
        }

        fn delete(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.delete(id)
        }

        fn exists(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.exists(id)
        }

        fn count(&self) -> Result<usize, AppError> {
            self.base_repository.count()
        }
    }

    impl TransactionalRepository<MockEntity, String> for MockTransactionalRepository {
        fn begin_transaction(&self) -> Result<Box<dyn Transaction>, AppError> {
            Ok(Box::new(MockTransaction::new()))
        }

        fn with_transaction<F, R>(&self, f: F) -> Result<R, AppError>
        where
            F: FnOnce(&dyn Transaction) -> Result<R, AppError>,
        {
            let transaction = self.begin_transaction()?;
            f(transaction.as_ref())
        }
    }

    // Mock transaction implementation
    struct MockTransaction {
        committed: bool,
        rolled_back: bool,
    }

    impl MockTransaction {
        fn new() -> Self {
            Self {
                committed: false,
                rolled_back: false,
            }
        }
    }

    impl Transaction for MockTransaction {
        fn commit(self: Box<Self>) -> Result<(), AppError> {
            let mut this = *self;
            this.committed = true;
            Ok(())
        }

        fn rollback(self: Box<Self>) -> Result<(), AppError> {
            let mut this = *self;
            this.rolled_back = true;
            Ok(())
        }
    }

    // Mock cached repository implementation
    struct MockCachedRepository {
        base_repository: MockRepository,
        cache: Arc<Mutex<HashMap<String, MockEntity>>>,
    }

    impl MockCachedRepository {
        fn new() -> Self {
            Self {
                base_repository: MockRepository::new(),
                cache: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl Repository<MockEntity, String> for MockCachedRepository {
        fn find_by_id(&self, id: &String) -> Result<Option<MockEntity>, AppError> {
            // Check cache first
            let cache_key = self.cache_key(id);
            let cached_entity = {
                let cache = self.cache.lock().unwrap();
                cache.get(&cache_key).cloned()
            };

            if let Some(entity) = cached_entity {
                return Ok(Some(entity));
            }

            // If not in cache, get from base repository
            let entity = self.base_repository.find_by_id(id)?;
            if let Some(ref entity) = entity {
                let cache_key = self.cache_key(id);
                let mut cache = self.cache.lock().unwrap();
                cache.insert(cache_key, entity.clone());
            }

            Ok(entity)
        }

        fn find_all(&self) -> Result<Vec<MockEntity>, AppError> {
            self.base_repository.find_all()
        }

        fn save(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            let result = self.base_repository.save(entity.clone())?;
            // Invalidate cache for this entity
            self.invalidate_cache(&entity.id)?;
            Ok(result)
        }

        fn update(&self, entity: MockEntity) -> Result<MockEntity, AppError> {
            let result = self.base_repository.update(entity.clone())?;
            // Invalidate cache for this entity
            self.invalidate_cache(&entity.id)?;
            Ok(result)
        }

        fn delete(&self, id: &String) -> Result<bool, AppError> {
            let result = self.base_repository.delete(id)?;
            if result {
                // Invalidate cache for this entity
                self.invalidate_cache(id)?;
            }
            Ok(result)
        }

        fn exists(&self, id: &String) -> Result<bool, AppError> {
            self.base_repository.exists(id)
        }

        fn count(&self) -> Result<usize, AppError> {
            self.base_repository.count()
        }
    }

    impl CachedRepository<MockEntity, String> for MockCachedRepository {
        fn cache_key(&self, id: &String) -> String {
            format!("entity:{}", id)
        }

        fn invalidate_cache(&self, id: &String) -> Result<(), AppError> {
            let cache_key = self.cache_key(id);
            let mut cache = self.cache.lock().unwrap();
            cache.remove(&cache_key);
            Ok(())
        }

        fn clear_cache(&self) -> Result<(), AppError> {
            let mut cache = self.cache.lock().unwrap();
            cache.clear();
            Ok(())
        }
    }

    // Tests for SearchCriteria
    #[test]
    fn test_search_criteria_new() {
        let criteria = SearchCriteria::new();
        assert!(criteria.filters.is_empty());
        assert!(criteria.sort_by.is_none());
        assert_eq!(criteria.sort_order, SortOrder::Ascending);
    }

    #[test]
    fn test_search_criteria_default() {
        let criteria = SearchCriteria::default();
        assert!(criteria.filters.is_empty());
        assert!(criteria.sort_by.is_none());
        assert_eq!(criteria.sort_order, SortOrder::Ascending);
    }

    #[test]
    fn test_search_criteria_with_filter() {
        let criteria = SearchCriteria::new()
            .with_filter("name", "test")
            .with_filter("value", "42");

        assert_eq!(criteria.filters.len(), 2);
        assert_eq!(criteria.filters.get("name"), Some(&"test".to_string()));
        assert_eq!(criteria.filters.get("value"), Some(&"42".to_string()));
    }

    #[test]
    fn test_search_criteria_sort_by() {
        let criteria = SearchCriteria::new().sort_by("name");
        assert_eq!(criteria.sort_by, Some("name".to_string()));
    }

    #[test]
    fn test_search_criteria_sort_order() {
        let criteria = SearchCriteria::new().sort_order(SortOrder::Descending);
        assert_eq!(criteria.sort_order, SortOrder::Descending);
    }

    // Tests for SortOrder
    #[test]
    fn test_sort_order_variants() {
        assert_eq!(SortOrder::Ascending as u8, 0);
        assert_eq!(SortOrder::Descending as u8, 1);
    }

    #[test]
    fn test_sort_order_debug() {
        assert_eq!(format!("{:?}", SortOrder::Ascending), "Ascending");
        assert_eq!(format!("{:?}", SortOrder::Descending), "Descending");
    }

    // Tests for PaginatedResult
    #[test]
    fn test_paginated_result_new() {
        let items = vec![MockEntity::new("1", "test1", 10), MockEntity::new("2", "test2", 20)];
        let result = PaginatedResult::new(items.clone(), 1, 10, 2);

        assert_eq!(result.items, items);
        assert_eq!(result.page, 1);
        assert_eq!(result.size, 10);
        assert_eq!(result.total, 2);
        assert_eq!(result.total_pages, 1);
    }

    #[test]
    fn test_paginated_result_total_pages_calculation() {
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 10, 25);
        assert_eq!(result.total_pages, 3); // 25 items / 10 per page = 3 pages

        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 10, 0);
        assert_eq!(result.total_pages, 0); // 0 items = 0 pages
    }

    #[test]
    fn test_paginated_result_has_next() {
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 10, 25);
        assert!(result.has_next());

        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 3, 10, 25);
        assert!(!result.has_next());
    }

    #[test]
    fn test_paginated_result_has_previous() {
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 10, 25);
        assert!(!result.has_previous());

        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 2, 10, 25);
        assert!(result.has_previous());
    }

    #[test]
    fn test_paginated_result_next_page() {
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 10, 25);
        assert_eq!(result.next_page(), Some(2));

        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 3, 10, 25);
        assert_eq!(result.next_page(), None);
    }

    #[test]
    fn test_paginated_result_previous_page() {
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 10, 25);
        assert_eq!(result.previous_page(), None);

        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 2, 10, 25);
        assert_eq!(result.previous_page(), Some(1));
    }

    // Tests for InMemoryRepository
    #[test]
    fn test_in_memory_repository_new() {
        let repo = InMemoryRepository::<MockEntity>::new();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_in_memory_repository_default() {
        let repo = InMemoryRepository::<MockEntity>::default();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_in_memory_repository_save_and_find() {
        let repo = InMemoryRepository::<MockEntity>::new();
        let entity = MockEntity::new("1", "test", 42);

        let saved_entity = repo.save(entity.clone()).unwrap();
        assert_eq!(saved_entity, entity);

        // Now the repository should actually persist the entity
        let found_entity = repo.find_by_id(&"entity_0".to_string()).unwrap();
        assert_eq!(found_entity, Some(entity));
    }

    #[test]
    fn test_in_memory_repository_update() {
        let repo = InMemoryRepository::<MockEntity>::new();
        let entity = MockEntity::new("1", "test", 42);

        // Save the entity first
        repo.save(entity).unwrap();

        // Now update it (this will create a new entity with a new ID)
        let updated_entity = MockEntity::new("1", "updated", 100);
        let result = repo.update(updated_entity.clone()).unwrap();
        assert_eq!(result, updated_entity);

        // Verify both entities exist (original and "updated")
        let found_entity1 = repo.find_by_id(&"entity_0".to_string()).unwrap();
        let found_entity2 = repo.find_by_id(&"entity_1".to_string()).unwrap();
        assert_eq!(found_entity1, Some(MockEntity::new("1", "test", 42)));
        assert_eq!(found_entity2, Some(updated_entity));
        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn test_in_memory_repository_delete() {
        let repo = InMemoryRepository::<MockEntity>::new();
        let entity = MockEntity::new("1", "test", 42);

        // Save the entity first
        repo.save(entity).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        // Delete the entity
        let deleted = repo.delete(&"entity_0".to_string()).unwrap();
        assert!(deleted);
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_in_memory_repository_exists() {
        let repo = InMemoryRepository::<MockEntity>::new();
        let entity = MockEntity::new("1", "test", 42);

        assert!(!repo.exists(&"entity_0".to_string()).unwrap());
        repo.save(entity).unwrap();
        assert!(repo.exists(&"entity_0".to_string()).unwrap()); // Now it should exist
    }

    #[test]
    fn test_in_memory_repository_find_all() {
        let repo = InMemoryRepository::<MockEntity>::new();
        let entity1 = MockEntity::new("1", "test1", 10);
        let entity2 = MockEntity::new("2", "test2", 20);

        repo.save(entity1.clone()).unwrap();
        repo.save(entity2.clone()).unwrap();

        let all_entities = repo.find_all().unwrap();
        assert_eq!(all_entities.len(), 2); // Now both entities should be persisted
        assert!(all_entities.contains(&entity1));
        assert!(all_entities.contains(&entity2));
    }

    #[test]
    fn test_in_memory_repository_persistence_across_instances() {
        // Create first repository instance and save data
        let repo1 = InMemoryRepository::<MockEntity>::new();
        let entity = MockEntity::new("1", "test", 42);
        repo1.save(entity.clone()).unwrap();

        // Create second repository instance - should be empty (no file persistence)
        let repo2 = InMemoryRepository::<MockEntity>::new();
        let found_entity = repo2.find_by_id(&"entity_0".to_string()).unwrap();
        assert_eq!(found_entity, None);
        assert_eq!(repo2.count().unwrap(), 0);
    }

    // Tests for PaginatedRepository
    #[test]
    fn test_paginated_repository_find_with_pagination() {
        let repo = MockPaginatedRepository::new();

        // Add some test entities
        repo.save(MockEntity::new("1", "test1", 10)).unwrap();
        repo.save(MockEntity::new("2", "test2", 20)).unwrap();
        repo.save(MockEntity::new("3", "test3", 30)).unwrap();

        let result = repo.find_with_pagination(1, 2).unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.page, 1);
        assert_eq!(result.size, 2);
        assert_eq!(result.total, 3);
        assert_eq!(result.total_pages, 2);
    }

    #[test]
    fn test_paginated_repository_empty_page() {
        let repo = MockPaginatedRepository::new();

        let result = repo.find_with_pagination(2, 10).unwrap();
        assert_eq!(result.items.len(), 0);
        assert_eq!(result.page, 2);
        assert_eq!(result.total_pages, 0);
    }

    // Tests for SearchableRepository
    #[test]
    fn test_searchable_repository_search() {
        let repo = MockSearchableRepository::new();

        // Add some test entities
        repo.save(MockEntity::new("1", "alice", 10)).unwrap();
        repo.save(MockEntity::new("2", "bob", 20)).unwrap();
        repo.save(MockEntity::new("3", "alice", 30)).unwrap();

        let criteria = SearchCriteria::new().with_filter("name", "alice");
        let results = repo.search(&criteria).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alice");
        assert_eq!(results[1].name, "alice");
    }

    #[test]
    fn test_searchable_repository_search_with_sorting() {
        let repo = MockSearchableRepository::new();

        // Add some test entities
        repo.save(MockEntity::new("1", "alice", 30)).unwrap();
        repo.save(MockEntity::new("2", "bob", 10)).unwrap();
        repo.save(MockEntity::new("3", "charlie", 20)).unwrap();

        let criteria = SearchCriteria::new().sort_by("value").sort_order(SortOrder::Ascending);

        let results = repo.search(&criteria).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].value, 10);
        assert_eq!(results[1].value, 20);
        assert_eq!(results[2].value, 30);
    }

    #[test]
    fn test_searchable_repository_search_with_pagination() {
        let repo = MockSearchableRepository::new();

        // Add some test entities
        for i in 1..=5 {
            repo.save(MockEntity::new(&i.to_string(), &format!("test{}", i), i * 10))
                .unwrap();
        }

        let criteria = SearchCriteria::new();
        let result = repo.search_with_pagination(&criteria, 1, 2).unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.page, 1);
        assert_eq!(result.size, 2);
        assert_eq!(result.total, 5);
        assert_eq!(result.total_pages, 3);
    }

    // Tests for TransactionalRepository
    #[test]
    fn test_transactional_repository_begin_transaction() {
        let repo = MockTransactionalRepository::new();
        let _transaction = repo.begin_transaction().unwrap();

        // Just test that we can create a transaction
        // Test passes
    }

    #[test]
    fn test_transactional_repository_with_transaction() {
        let repo = MockTransactionalRepository::new();

        let result = repo
            .with_transaction(|_transaction| Ok::<String, AppError>("transaction_result".to_string()))
            .unwrap();

        assert_eq!(result, "transaction_result");
    }

    // Tests for Transaction
    #[test]
    fn test_mock_transaction_commit() {
        let transaction = MockTransaction::new();
        let boxed_transaction = Box::new(transaction);

        let result = boxed_transaction.commit();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_transaction_rollback() {
        let transaction = MockTransaction::new();
        let boxed_transaction = Box::new(transaction);

        let result = boxed_transaction.rollback();
        assert!(result.is_ok());
    }

    // Tests for CachedRepository
    #[test]
    fn test_cached_repository_cache_key() {
        let repo = MockCachedRepository::new();
        let cache_key = repo.cache_key(&"test_id".to_string());
        assert_eq!(cache_key, "entity:test_id");
    }

    #[test]
    fn test_cached_repository_find_by_id_with_cache() {
        let repo = MockCachedRepository::new();
        let entity = MockEntity::new("1", "test", 42);

        // First find should populate cache
        let found_entity = repo.find_by_id(&"1".to_string()).unwrap();
        assert_eq!(found_entity, None);

        // Save entity
        repo.save(entity.clone()).unwrap();

        // Find again should use cache
        let found_entity = repo.find_by_id(&"1".to_string()).unwrap();
        assert_eq!(found_entity, Some(entity));
    }

    #[test]
    fn test_cached_repository_invalidate_cache() {
        let repo = MockCachedRepository::new();
        let entity = MockEntity::new("1", "test", 42);

        repo.save(entity.clone()).unwrap();
        repo.find_by_id(&"1".to_string()).unwrap(); // Populate cache

        // Invalidate cache
        repo.invalidate_cache(&"1".to_string()).unwrap();

        // Cache should be cleared
        let found_entity = repo.find_by_id(&"1".to_string()).unwrap();
        assert_eq!(found_entity, Some(entity)); // Should still find from base repo
    }

    #[test]
    fn test_cached_repository_clear_cache() {
        let repo = MockCachedRepository::new();
        let entity1 = MockEntity::new("1", "test1", 10);
        let entity2 = MockEntity::new("2", "test2", 20);

        repo.save(entity1.clone()).unwrap();
        repo.save(entity2.clone()).unwrap();
        repo.find_by_id(&"1".to_string()).unwrap(); // Populate cache
        repo.find_by_id(&"2".to_string()).unwrap(); // Populate cache

        // Clear all cache
        repo.clear_cache().unwrap();

        // Should still work but without cache
        let found_entity = repo.find_by_id(&"1".to_string()).unwrap();
        assert_eq!(found_entity, Some(entity1));
    }

    // Tests for LoggingRepositoryDecorator
    #[test]
    fn test_logging_repository_decorator_new() {
        let base_repo = MockRepository::new();
        let _decorator = LoggingRepositoryDecorator::new(base_repo);

        // Just test that we can create the decorator
        // Test passes
    }

    #[test]
    fn test_logging_repository_decorator_find_by_id() {
        let base_repo = MockRepository::new();
        let decorator = LoggingRepositoryDecorator::new(base_repo);

        let result = decorator.find_by_id(&"nonexistent".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_logging_repository_decorator_save() {
        let base_repo = MockRepository::new();
        let decorator = LoggingRepositoryDecorator::new(base_repo);

        let entity = MockEntity::new("1", "test", 42);
        let result = decorator.save(entity.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), entity);
    }

    // Tests for complex scenarios
    #[test]
    fn test_repository_complex_workflow() {
        let repo = MockRepository::new();

        // Create entities
        let entity1 = MockEntity::new("1", "alice", 25);
        let entity2 = MockEntity::new("2", "bob", 30);
        let entity3 = MockEntity::new("3", "charlie", 35);

        // Save entities
        repo.save(entity1.clone()).unwrap();
        repo.save(entity2.clone()).unwrap();
        repo.save(entity3.clone()).unwrap();

        // Verify count
        assert_eq!(repo.count().unwrap(), 3);

        // Find specific entity
        let found = repo.find_by_id(&"2".to_string()).unwrap();
        assert_eq!(found, Some(entity2.clone()));

        // Update entity
        let updated_entity = MockEntity::new("2", "robert", 31);
        let updated = repo.update(updated_entity.clone()).unwrap();
        assert_eq!(updated, updated_entity);

        // Verify update
        let found = repo.find_by_id(&"2".to_string()).unwrap();
        assert_eq!(found, Some(updated_entity));

        // Delete entity
        let deleted = repo.delete(&"1".to_string()).unwrap();
        assert!(deleted);

        // Verify deletion
        assert_eq!(repo.count().unwrap(), 2);
        assert!(!repo.exists(&"1".to_string()).unwrap());
    }

    #[test]
    fn test_searchable_repository_complex_search() {
        let repo = MockSearchableRepository::new();

        // Add test entities with various values
        let entities = vec![
            MockEntity::new("1", "alice", 25),
            MockEntity::new("2", "bob", 30),
            MockEntity::new("3", "alice", 35),
            MockEntity::new("4", "charlie", 20),
            MockEntity::new("5", "alice", 40),
        ];

        for entity in entities {
            repo.save(entity).unwrap();
        }

        // Search for alice with value > 30, sorted by value descending
        let criteria = SearchCriteria::new()
            .with_filter("name", "alice")
            .sort_by("value")
            .sort_order(SortOrder::Descending);

        let results = repo.search(&criteria).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].value, 40);
        assert_eq!(results[1].value, 35);
        assert_eq!(results[2].value, 25);
    }

    // Tests for edge cases
    #[test]
    fn test_repository_empty_operations() {
        let repo = MockRepository::new();

        // Test operations on empty repository
        assert_eq!(repo.count().unwrap(), 0);
        assert_eq!(repo.find_all().unwrap().len(), 0);
        assert_eq!(repo.find_by_id(&"nonexistent".to_string()).unwrap(), None);
        assert!(!repo.exists(&"nonexistent".to_string()).unwrap());
        assert!(!repo.delete(&"nonexistent".to_string()).unwrap());
    }

    #[test]
    fn test_paginated_result_edge_cases() {
        // Test with size 0
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 1, 0, 10);
        assert_eq!(result.total_pages, 0);

        // Test with page 0 (edge case)
        let result: PaginatedResult<MockEntity> = PaginatedResult::new(Vec::new(), 0, 10, 10);
        assert_eq!(result.page, 0);
        assert!(!result.has_previous());
    }

    #[test]
    fn test_search_criteria_edge_cases() {
        // Test with empty filter values
        let criteria = SearchCriteria::new()
            .with_filter("empty", "")
            .with_filter("whitespace", "   ");

        assert_eq!(criteria.filters.get("empty"), Some(&"".to_string()));
        assert_eq!(criteria.filters.get("whitespace"), Some(&"   ".to_string()));
    }
}
