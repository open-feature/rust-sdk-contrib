//! # Cache Service for Feature Flags
//!
//! Provides configurable caching functionality for feature flag values.
//!
//! ## Features
//!
//! * Thread-safe cache operations
//! * Multiple cache implementations
//! * TTL-based invalidation
//! * Size-bounded caching
//!
//! ## Cache Types
//!
//! * [`CacheType::Lru`] - Least Recently Used cache
//! * [`CacheType::InMemory`] - Simple in-memory cache
//! * [`CacheType::Disabled`] - No caching
//!
//! ## Example
//!
//! ```rust
//! use open_feature_flagd::cache::{CacheSettings, CacheType};
//! use std::time::Duration;
//!
//! let settings = CacheSettings {
//!     cache_type: CacheType::Lru,
//!     max_size: 1000,
//!     ttl: Some(Duration::from_secs(60)),
//! };
//! ```

use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum CacheType {
    Lru,
    InMemory,
    Disabled,
}

impl<'a> From<&'a str> for CacheType {
    fn from(s: &'a str) -> Self {
        match s.to_lowercase().as_str() {
            "lru" => CacheType::Lru,
            "mem" => CacheType::InMemory,
            "disabled" => CacheType::Disabled,
            _ => CacheType::Lru,
        }
    }
}

impl std::fmt::Display for CacheType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheType::Lru => write!(f, "lru"),
            CacheType::InMemory => write!(f, "mem"),
            CacheType::Disabled => write!(f, "disabled"),
        }
    }
}

/// Settings for configuring the cache behavior
#[derive(Debug, Clone)]
pub struct CacheSettings {
    /// Type of cache to use (LRU, InMemory, or Disabled)
    /// Default: LRU
    pub cache_type: CacheType,
    /// Maximum number of entries the cache can hold
    /// Default: 1000
    pub max_size: usize,
    /// Optional time-to-live for cache entries
    /// Default: 60 seconds
    pub ttl: Option<Duration>,
}

impl Default for CacheSettings {
    fn default() -> Self {
        let cache_type = std::env::var("FLAGD_CACHE")
            .map(|s| CacheType::from(s.as_str()))
            .unwrap_or(CacheType::Lru);

        let max_size = std::env::var("FLAGD_MAX_CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        // Java or Golang implementation does not use a default TTL, however
        // if there is no TTL cache is never expired, resulting in flag resolution
        // not updated. 60 second as a default is taken from gofeatureflag implementation.
        let ttl = std::env::var("FLAGD_CACHE_TTL")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .or_else(|| Some(Duration::from_secs(60)));

        Self {
            cache_type,
            max_size,
            ttl,
        }
    }
}

/// Entry in the cache with timestamp for TTL tracking
#[derive(Debug)]
struct CacheEntry<V>
where
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    value: V,
    created_at: Instant,
}

/// Core trait defining cache behavior
pub trait Cache<K, V>: Send + Sync + std::fmt::Debug {
    /// Adds a new key-value pair to the cache
    fn add(&mut self, key: K, value: V) -> bool;
    /// Removes all entries from the cache
    #[allow(dead_code)]
    fn purge(&mut self);
    /// Retrieves a value by key
    fn get(&mut self, key: &K) -> Option<&V>;
    /// Removes a specific key from the cache
    fn remove(&mut self, key: &K) -> bool;
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct CacheKey {
    flag_key: String,
    context_hash: String,
}

impl CacheKey {
    pub fn new(flag_key: &str, context: &EvaluationContext) -> Self {
        let mut hasher = DefaultHasher::new();
        // Hash targeting key if present
        if let Some(key) = &context.targeting_key {
            key.hash(&mut hasher);
        }
        // Hash custom fields
        for (key, value) in &context.custom_fields {
            key.hash(&mut hasher);
            match value {
                EvaluationContextFieldValue::String(s) => s.hash(&mut hasher),
                EvaluationContextFieldValue::Bool(b) => b.hash(&mut hasher),
                EvaluationContextFieldValue::Int(i) => i.hash(&mut hasher),
                EvaluationContextFieldValue::Float(f) => f.to_bits().hash(&mut hasher),
                EvaluationContextFieldValue::DateTime(dt) => dt.to_string().hash(&mut hasher),
                EvaluationContextFieldValue::Struct(s) => format!("{:?}", s).hash(&mut hasher),
            }
        }
        Self {
            flag_key: flag_key.to_string(),
            context_hash: hasher.finish().to_string(),
        }
    }
}

/// Type alias for the thread-safe cache implementation
type SharedCache<V> = Arc<RwLock<Box<dyn Cache<CacheKey, CacheEntry<V>>>>>;

/// Service managing cache operations and lifecycle
#[derive(Debug)]
pub struct CacheService<V>
where
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    /// Whether the cache is currently enabled
    enabled: bool,
    /// Time-to-live configuration for cache entries
    ttl: Option<Duration>,
    /// The underlying cache implementation
    cache: SharedCache<V>,
}

impl<V> CacheService<V>
where
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub fn new(settings: CacheSettings) -> Self {
        let (enabled, cache) = match settings.cache_type {
            CacheType::Lru => {
                let lru = crate::cache::lru::LruCacheImpl::new(settings.max_size);
                (
                    true,
                    Box::new(lru) as Box<dyn Cache<CacheKey, CacheEntry<V>>>,
                )
            }
            CacheType::InMemory => {
                let mem = crate::cache::in_memory::InMemoryCache::new();
                (
                    true,
                    Box::new(mem) as Box<dyn Cache<CacheKey, CacheEntry<V>>>,
                )
            }
            CacheType::Disabled => {
                let mem = crate::cache::in_memory::InMemoryCache::new();
                (
                    false,
                    Box::new(mem) as Box<dyn Cache<CacheKey, CacheEntry<V>>>,
                )
            }
        };

        Self {
            enabled,
            ttl: settings.ttl,
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub async fn get(&self, flag_key: &str, context: &EvaluationContext) -> Option<V> {
        if !self.enabled {
            return None;
        }

        let cache_key = CacheKey::new(flag_key, context);
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get(&cache_key) {
            if let Some(ttl) = self.ttl
                && entry.created_at.elapsed() > ttl
            {
                cache.remove(&cache_key);
                return None;
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub async fn add(&self, flag_key: &str, context: &EvaluationContext, value: V) -> bool {
        if !self.enabled {
            return false;
        }
        let cache_key = CacheKey::new(flag_key, context);
        let mut cache = self.cache.write().await;
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
        };
        cache.add(cache_key, entry)
    }

    pub async fn purge(&self) {
        if self.enabled {
            let mut cache = self.cache.write().await;
            cache.purge();
        }
    }

    pub fn disable(&mut self) {
        if self.enabled {
            self.enabled = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_cache_service_lru() {
        let settings = CacheSettings {
            cache_type: CacheType::Lru,
            max_size: 2,
            ttl: None,
        };
        let service = CacheService::<String>::new(settings);

        let context1 = EvaluationContext::default()
            .with_targeting_key("user1")
            .with_custom_field("email", "test1@example.com");

        let context2 = EvaluationContext::default()
            .with_targeting_key("user2")
            .with_custom_field("email", "test2@example.com");

        service.add("key1", &context1, "value1".to_string()).await;
        service.add("key1", &context2, "value2".to_string()).await;

        assert_eq!(
            service.get("key1", &context1).await,
            Some("value1".to_string())
        );
        assert_eq!(
            service.get("key1", &context2).await,
            Some("value2".to_string())
        );
    }

    #[test(tokio::test)]
    async fn test_cache_service_ttl() {
        let settings = CacheSettings {
            cache_type: CacheType::InMemory,
            max_size: 10,
            ttl: Some(Duration::from_secs(1)),
        };
        let service = CacheService::<String>::new(settings);

        let context = EvaluationContext::default()
            .with_targeting_key("user1")
            .with_custom_field("version", "1.0.0");

        service.add("key1", &context, "value1".to_string()).await;
        assert_eq!(
            service.get("key1", &context).await,
            Some("value1".to_string())
        );

        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(service.get("key1", &context).await, None);
    }

    #[test(tokio::test)]
    async fn test_cache_service_disabled() {
        let settings = CacheSettings {
            cache_type: CacheType::Disabled,
            max_size: 2,
            ttl: None,
        };
        let service = CacheService::<String>::new(settings);

        let context = EvaluationContext::default().with_targeting_key("user1");

        service.add("key1", &context, "value1".to_string()).await;
        assert_eq!(service.get("key1", &context).await, None);
    }

    #[test(tokio::test)]
    async fn test_different_contexts_same_flag() {
        let settings = CacheSettings {
            cache_type: CacheType::InMemory,
            max_size: 10,
            ttl: None,
        };
        let service = CacheService::<String>::new(settings);

        let context1 = EvaluationContext::default()
            .with_targeting_key("user1")
            .with_custom_field("email", "test1@example.com");

        let context2 = EvaluationContext::default()
            .with_targeting_key("user1")
            .with_custom_field("email", "test2@example.com");

        service
            .add("feature-flag", &context1, "variant1".to_string())
            .await;
        service
            .add("feature-flag", &context2, "variant2".to_string())
            .await;

        assert_eq!(
            service.get("feature-flag", &context1).await,
            Some("variant1".to_string())
        );
        assert_eq!(
            service.get("feature-flag", &context2).await,
            Some("variant2".to_string())
        );
    }
}
