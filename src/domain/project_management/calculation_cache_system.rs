//! Sistema de Cache para Cálculos
//!
//! Este módulo implementa um sistema robusto de cache para cálculos de dependências,
//! incluindo invalidação inteligente, persistência e otimização de performance.

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};

use super::advanced_dependencies::AdvancedDependency;
use super::dependency_calculation_engine::CalculationResult;
use crate::application::errors::AppError;

// ============================================================================
// ENUMS
// ============================================================================

/// Estratégia de invalidação do cache
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheInvalidationStrategy {
    /// Invalidação manual
    Manual,
    /// Invalidação automática baseada em mudanças
    Automatic,
    /// Invalidação baseada em tempo (TTL)
    TimeBased(Duration),
    /// Invalidação baseada em versão
    VersionBased,
}

/// Status de um item no cache
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheItemStatus {
    /// Item válido e pronto para uso
    Valid,
    /// Item expirado
    Expired,
    /// Item inválido
    Invalid,
    /// Item sendo calculado
    Calculating,
}

/// Tipo de operação no cache
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheOperation {
    /// Operação de leitura
    Read,
    /// Operação de escrita
    Write,
    /// Operação de invalidação
    Invalidate,
    /// Operação de limpeza
    Clear,
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Item do cache com metadados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheItem<T> {
    pub data: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub version: u64,
    pub status: CacheItemStatus,
    pub ttl: Option<Duration>,
    pub dependencies: HashSet<String>,
}

impl<T> CacheItem<T> {
    /// Cria um novo item do cache
    pub fn new(data: T, version: u64) -> Self {
        let now = chrono::Utc::now();
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            version,
            status: CacheItemStatus::Valid,
            ttl: None,
            dependencies: HashSet::new(),
        }
    }

    /// Verifica se o item está expirado
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = chrono::Utc::now();
            now.signed_duration_since(self.created_at) > ttl
        } else {
            false
        }
    }

    /// Atualiza o acesso ao item
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
        self.access_count += 1;
    }

    /// Adiciona uma dependência
    pub fn add_dependency(&mut self, dep: String) {
        self.dependencies.insert(dep);
    }

    /// Remove uma dependência
    pub fn remove_dependency(&mut self, dep: &str) {
        self.dependencies.remove(dep);
    }
}

/// Chave do cache baseada em hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub task_id: String,
    pub dependency_hash: String,
    pub config_hash: String,
    pub version: u64,
}

impl CacheKey {
    /// Cria uma nova chave do cache
    pub fn new(task_id: String, dependency_hash: String, config_hash: String, version: u64) -> Self {
        Self {
            task_id,
            dependency_hash,
            config_hash,
            version,
        }
    }

    /// Gera hash das dependências
    pub fn hash_dependencies(dependencies: &[AdvancedDependency]) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for dep in dependencies {
            dep.id.hash(&mut hasher);
            dep.dependency_type.hash(&mut hasher);
            dep.lag.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Gera hash da configuração
    pub fn hash_config(config: &super::dependency_calculation_engine::CalculationConfig) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        config.project_start_date.hash(&mut hasher);
        config.default_task_duration.hash(&mut hasher);
        config.working_days_only.hash(&mut hasher);
        config.working_hours_per_day.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Estatísticas do cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_items: usize,
    pub valid_items: usize,
    pub expired_items: usize,
    pub invalid_items: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub memory_usage: usize,
    pub oldest_item: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_item: Option<chrono::DateTime<chrono::Utc>>,
}

/// Configuração do sistema de cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Tamanho máximo do cache (número de itens)
    pub max_size: usize,
    /// TTL padrão para itens do cache
    pub default_ttl: Option<Duration>,
    /// Estratégia de invalidação
    pub invalidation_strategy: CacheInvalidationStrategy,
    /// Cache habilitado
    pub enabled: bool,
    /// Limpeza automática habilitada
    pub auto_cleanup: bool,
    /// Intervalo de limpeza automática
    pub cleanup_interval: Duration,
    /// Persistência habilitada
    pub persistence_enabled: bool,
    /// Caminho para persistência
    pub persistence_path: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: Some(Duration::hours(1)),
            invalidation_strategy: CacheInvalidationStrategy::Automatic,
            enabled: true,
            auto_cleanup: true,
            cleanup_interval: Duration::minutes(30),
            persistence_enabled: false,
            persistence_path: None,
        }
    }
}

/// Sistema de cache para cálculos
#[derive(Debug, Clone)]
pub struct CalculationCacheSystem {
    config: CacheConfig,
    cache: HashMap<CacheKey, CacheItem<CalculationResult>>,
    dependency_map: HashMap<String, HashSet<CacheKey>>,
    version: u64,
    stats: CacheStats,
    last_cleanup: chrono::DateTime<chrono::Utc>,
}

impl CalculationCacheSystem {
    /// Cria um novo sistema de cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            dependency_map: HashMap::new(),
            version: 1,
            stats: CacheStats {
                total_items: 0,
                valid_items: 0,
                expired_items: 0,
                invalid_items: 0,
                hit_count: 0,
                miss_count: 0,
                hit_rate: 0.0,
                memory_usage: 0,
                oldest_item: None,
                newest_item: None,
            },
            last_cleanup: chrono::Utc::now(),
        }
    }

    /// Cria um sistema com configuração padrão
    pub fn with_default_config() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Obtém um resultado do cache
    pub fn get(
        &mut self,
        task_id: &str,
        dependencies: &[AdvancedDependency],
        config: &super::dependency_calculation_engine::CalculationConfig,
    ) -> Option<CalculationResult> {
        if !self.config.enabled {
            return None;
        }

        let key = self.create_cache_key(task_id, dependencies, config);

        if let Some(item) = self.cache.get_mut(&key) {
            // Verificar se o item é válido
            if item.status == CacheItemStatus::Valid && !item.is_expired() {
                item.touch();
                self.stats.hit_count += 1;
                let data = item.data.clone();
                self.update_hit_rate();
                return Some(data);
            } else {
                // Item inválido ou expirado
                item.status = CacheItemStatus::Invalid;
                self.stats.miss_count += 1;
                self.update_hit_rate();
            }
        } else {
            self.stats.miss_count += 1;
            self.update_hit_rate();
        }

        None
    }

    /// Armazena um resultado no cache
    pub fn put(
        &mut self,
        task_id: &str,
        dependencies: &[AdvancedDependency],
        config: &super::dependency_calculation_engine::CalculationConfig,
        result: CalculationResult,
    ) -> Result<(), AppError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Verificar se o cache está cheio
        if self.cache.len() >= self.config.max_size {
            self.evict_oldest_items(1)?;
        }

        let key = self.create_cache_key(task_id, dependencies, config);
        let mut item = CacheItem::new(result, self.version);
        item.ttl = self.config.default_ttl;

        // Adicionar dependências
        for dep in dependencies {
            item.add_dependency(dep.id.clone());
        }

        // Armazenar no cache
        self.cache.insert(key.clone(), item);
        self.stats.total_items = self.cache.len();

        // Atualizar mapa de dependências
        for dep in dependencies {
            self.dependency_map
                .entry(dep.id.clone())
                .or_default()
                .insert(key.clone());
        }

        // Atualizar estatísticas
        self.update_stats();

        Ok(())
    }

    /// Invalida itens do cache baseado em uma dependência
    pub fn invalidate_by_dependency(&mut self, dependency_id: &str) -> Result<usize, AppError> {
        if !self.config.enabled {
            return Ok(0);
        }

        let mut invalidated = 0;

        if let Some(keys) = self.dependency_map.get(dependency_id) {
            for key in keys.clone() {
                if let Some(item) = self.cache.get_mut(&key) {
                    item.status = CacheItemStatus::Invalid;
                    invalidated += 1;
                }
            }
        }

        // Remover itens inválidos
        self.cleanup_invalid_items();

        Ok(invalidated)
    }

    /// Invalida itens do cache baseado em uma tarefa
    pub fn invalidate_by_task(&mut self, task_id: &str) -> Result<usize, AppError> {
        if !self.config.enabled {
            return Ok(0);
        }

        let mut invalidated = 0;
        let keys_to_remove: Vec<CacheKey> = self
            .cache
            .keys()
            .filter(|key| key.task_id == task_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            if self.cache.remove(&key).is_some() {
                invalidated += 1;
            }
        }

        self.update_stats();
        Ok(invalidated)
    }

    /// Invalida todo o cache
    pub fn invalidate_all(&mut self) -> Result<usize, AppError> {
        if !self.config.enabled {
            return Ok(0);
        }

        let count = self.cache.len();
        self.cache.clear();
        self.dependency_map.clear();
        self.version += 1;
        self.update_stats();

        Ok(count)
    }

    /// Limpa itens expirados e inválidos
    pub fn cleanup(&mut self) -> Result<usize, AppError> {
        if !self.config.enabled {
            return Ok(0);
        }

        let mut cleaned = 0;
        let now = chrono::Utc::now();

        // Verificar se é hora da limpeza automática
        if self.config.auto_cleanup && now.signed_duration_since(self.last_cleanup) > self.config.cleanup_interval {
            cleaned += self.cleanup_expired_items()?;
            cleaned += self.cleanup_invalid_items();
            self.last_cleanup = now;
        }

        Ok(cleaned)
    }

    /// Cria uma chave do cache
    fn create_cache_key(
        &self,
        task_id: &str,
        dependencies: &[AdvancedDependency],
        config: &super::dependency_calculation_engine::CalculationConfig,
    ) -> CacheKey {
        let dependency_hash = CacheKey::hash_dependencies(dependencies);
        let config_hash = CacheKey::hash_config(config);

        CacheKey::new(task_id.to_string(), dependency_hash, config_hash, self.version)
    }

    /// Remove itens mais antigos do cache
    fn evict_oldest_items(&mut self, count: usize) -> Result<(), AppError> {
        let mut items: Vec<(CacheKey, chrono::DateTime<chrono::Utc>)> = self
            .cache
            .iter()
            .map(|(key, item)| (key.clone(), item.last_accessed))
            .collect();

        items.sort_by(|a, b| a.1.cmp(&b.1));

        for (key, _) in items.iter().take(count) {
            self.cache.remove(key);
        }

        self.update_stats();
        Ok(())
    }

    /// Limpa itens expirados
    fn cleanup_expired_items(&mut self) -> Result<usize, AppError> {
        let mut expired_keys = Vec::new();

        for (key, item) in &self.cache {
            if item.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        let count = expired_keys.len();
        for key in expired_keys {
            self.cache.remove(&key);
        }

        self.update_stats();
        Ok(count)
    }

    /// Limpa itens inválidos
    fn cleanup_invalid_items(&mut self) -> usize {
        let mut invalid_keys = Vec::new();

        for (key, item) in &self.cache {
            if item.status == CacheItemStatus::Invalid {
                invalid_keys.push(key.clone());
            }
        }

        let count = invalid_keys.len();
        for key in invalid_keys {
            self.cache.remove(&key);
        }

        self.update_stats();
        count
    }

    /// Atualiza estatísticas do cache
    fn update_stats(&mut self) {
        self.stats.total_items = self.cache.len();
        self.stats.valid_items = self
            .cache
            .values()
            .filter(|item| item.status == CacheItemStatus::Valid)
            .count();
        self.stats.expired_items = self.cache.values().filter(|item| item.is_expired()).count();
        self.stats.invalid_items = self
            .cache
            .values()
            .filter(|item| item.status == CacheItemStatus::Invalid)
            .count();

        // Calcular memória usada (estimativa)
        self.stats.memory_usage = self.cache.len() * std::mem::size_of::<CacheItem<CalculationResult>>();

        // Encontrar item mais antigo e mais novo
        self.stats.oldest_item = self.cache.values().map(|item| item.created_at).min();
        self.stats.newest_item = self.cache.values().map(|item| item.created_at).max();
    }

    /// Atualiza taxa de acerto
    fn update_hit_rate(&mut self) {
        let total = self.stats.hit_count + self.stats.miss_count;
        if total > 0 {
            self.stats.hit_rate = self.stats.hit_count as f64 / total as f64;
        }
    }

    /// Obtém estatísticas do cache
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Obtém configuração do cache
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    /// Atualiza configuração do cache
    pub fn update_config(&mut self, config: CacheConfig) {
        self.config = config;
        if !self.config.enabled {
            self.cache.clear();
            self.dependency_map.clear();
        }
    }

    /// Obtém informações de debug do cache
    pub fn debug_info(&self) -> String {
        format!(
            "Cache: {} items, {:.2}% hit rate, {} memory",
            self.stats.total_items,
            self.stats.hit_rate * 100.0,
            self.stats.memory_usage
        )
    }
}

impl fmt::Display for CacheStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cache Stats: {} items ({} valid, {} expired, {} invalid), {:.2}% hit rate, {} bytes",
            self.total_items,
            self.valid_items,
            self.expired_items,
            self.invalid_items,
            self.hit_rate * 100.0,
            self.memory_usage
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::{DependencyType, LagType};
    use chrono::NaiveDate;

    fn create_test_dependency() -> AdvancedDependency {
        AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        )
    }

    fn create_test_result() -> CalculationResult {
        CalculationResult {
            task_id: "task1".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: false,
            total_float: Some(Duration::days(0)),
            free_float: None,
            dependencies_satisfied: true,
            calculation_order: 0,
        }
    }

    #[test]
    fn test_cache_system_creation() {
        let cache = CalculationCacheSystem::with_default_config();
        assert_eq!(cache.get_stats().total_items, 0);
        assert_eq!(cache.get_stats().hit_rate, 0.0);
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = CalculationCacheSystem::with_default_config();
        let dependency = create_test_dependency();
        let result = create_test_result();
        let config = super::super::dependency_calculation_engine::CalculationConfig::default();

        // Armazenar no cache
        cache
            .put("task1", std::slice::from_ref(&dependency), &config, result.clone())
            .unwrap();

        // Recuperar do cache
        let retrieved = cache.get("task1", &[dependency], &config);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().task_id, "task1");
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = CalculationCacheSystem::with_default_config();
        let dependency = create_test_dependency();
        let config = super::super::dependency_calculation_engine::CalculationConfig::default();

        // Tentar recuperar item que não existe
        let retrieved = cache.get("nonexistent", &[dependency], &config);
        assert!(retrieved.is_none());
        assert_eq!(cache.get_stats().miss_count, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = CalculationCacheSystem::with_default_config();
        let dependency = create_test_dependency();
        let result = create_test_result();
        let config = super::super::dependency_calculation_engine::CalculationConfig::default();

        // Armazenar no cache
        cache
            .put("task1", std::slice::from_ref(&dependency), &config, result)
            .unwrap();

        // Invalidar por dependência
        let invalidated = cache.invalidate_by_dependency(&dependency.id).unwrap();
        assert_eq!(invalidated, 1);

        // Tentar recuperar (deve falhar)
        let retrieved = cache.get("task1", &[dependency], &config);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = CalculationCacheSystem::with_default_config();
        let dependency = create_test_dependency();
        let result = create_test_result();
        let config = super::super::dependency_calculation_engine::CalculationConfig::default();

        // Armazenar no cache
        cache.put("task1", &[dependency], &config, result).unwrap();

        // Limpar cache
        let cleaned = cache.cleanup().unwrap();
        assert_eq!(cleaned, 0); // Nenhum item expirado ainda

        // Invalidar todo o cache
        let invalidated = cache.invalidate_all().unwrap();
        assert_eq!(invalidated, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = CalculationCacheSystem::with_default_config();
        let dependency = create_test_dependency();
        let result = create_test_result();
        let config = super::super::dependency_calculation_engine::CalculationConfig::default();

        // Armazenar no cache
        cache
            .put("task1", std::slice::from_ref(&dependency), &config, result)
            .unwrap();

        // Recuperar do cache
        cache.get("task1", &[dependency], &config);

        let stats = cache.get_stats();
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.hit_rate, 1.0);
    }

    #[test]
    fn test_cache_config_update() {
        let mut cache = CalculationCacheSystem::with_default_config();
        let dependency = create_test_dependency();
        let result = create_test_result();
        let config = super::super::dependency_calculation_engine::CalculationConfig::default();

        // Armazenar no cache
        cache
            .put("task1", std::slice::from_ref(&dependency), &config, result)
            .unwrap();

        // Desabilitar cache
        let mut new_config = cache.get_config().clone();
        new_config.enabled = false;
        cache.update_config(new_config);

        // Tentar recuperar (deve falhar pois cache está desabilitado)
        let retrieved = cache.get("task1", &[dependency], &config);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_cache_stats_display() {
        let stats = CacheStats {
            total_items: 10,
            valid_items: 8,
            expired_items: 1,
            invalid_items: 1,
            hit_count: 50,
            miss_count: 10,
            hit_rate: 0.833,
            memory_usage: 1024,
            oldest_item: Some(chrono::Utc::now()),
            newest_item: Some(chrono::Utc::now()),
        };

        let display = format!("{}", stats);
        assert!(display.contains("10 items"));
        assert!(display.contains("83.30% hit rate"));
        assert!(display.contains("1024 bytes"));
    }
}
