//! # In-Memory Cache Implementation
//!
//! Simple HashMap-based cache for feature flag values.
//!
//! ## Features
//!
//! * Fast lookups
//! * Simple implementation
//! * No eviction policy
//! * Thread-safe operations

use super::service::Cache;
use std::collections::HashMap;
use std::hash::Hash;

/// Simple in-memory cache implementation using a HashMap
#[derive(Debug)]
pub struct InMemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + std::fmt::Debug,
    V: Send + Sync + std::fmt::Debug,
{
    /// Internal storage for cached values
    cache: HashMap<K, V>,
}

impl<K, V> InMemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + std::fmt::Debug,
    V: Send + Sync + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<K, V> Cache<K, V> for InMemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + std::fmt::Debug,
    V: Send + Sync + std::fmt::Debug,
{
    fn add(&mut self, key: K, value: V) -> bool {
        self.cache.insert(key, value).is_some()
    }

    fn purge(&mut self) {
        self.cache.clear();
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    fn remove(&mut self, key: &K) -> bool {
        self.cache.remove(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_cache_operations() {
        let mut cache = InMemoryCache::<String, i32>::new();

        assert_eq!(cache.add("key1".to_string(), 1), false);
        assert_eq!(cache.get(&"key1".to_string()), Some(&1));

        assert_eq!(cache.remove(&"key1".to_string()), true);
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_in_memory_cache_purge() {
        let mut cache = InMemoryCache::<String, i32>::new();

        cache.add("key1".to_string(), 1);
        cache.add("key2".to_string(), 2);

        cache.purge();
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), None);
    }
}
