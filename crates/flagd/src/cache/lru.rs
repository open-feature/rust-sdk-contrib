//! # LRU Cache Implementation
//!
//! Provides a size-bounded Least Recently Used (LRU) cache for feature flag values.
//!
//! ## Features
//!
//! * Constant memory usage
//! * O(1) operations
//! * Automatic eviction of least used entries
//! * Thread-safe operations

use super::service::Cache;
use lru::LruCache;
use std::hash::Hash;

/// LRU cache implementation with bounded size
#[derive(Debug)]
pub struct LruCacheImpl<K, V>
where
    K: Hash + Eq + Send + Sync + std::fmt::Debug,
    V: Send + Sync + std::fmt::Debug,
{
    cache: LruCache<K, V>,
}

impl<K, V> LruCacheImpl<K, V>
where
    K: Hash + Eq + Send + Sync + std::fmt::Debug,
    V: Send + Sync + std::fmt::Debug,
{
    pub fn new(size: usize) -> Self {
        Self {
            cache: LruCache::new(size.try_into().unwrap()),
        }
    }
}

impl<K, V> Cache<K, V> for LruCacheImpl<K, V>
where
    K: Hash + Eq + Send + Sync + std::fmt::Debug,
    V: Send + Sync + std::fmt::Debug,
{
    fn add(&mut self, key: K, value: V) -> bool {
        self.cache.put(key, value).is_some()
    }

    fn purge(&mut self) {
        self.cache.clear();
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    fn remove(&mut self, key: &K) -> bool {
        self.cache.pop(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_operations() {
        let mut cache = LruCacheImpl::<String, i32>::new(2);

        assert_eq!(cache.add("key1".to_string(), 1), false);
        assert_eq!(cache.get(&"key1".to_string()), Some(&1));

        assert_eq!(cache.remove(&"key1".to_string()), true);
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_lru_cache_capacity() {
        let mut cache = LruCacheImpl::<String, i32>::new(2);

        cache.add("key1".to_string(), 1);
        cache.add("key2".to_string(), 2);
        cache.add("key3".to_string(), 3);

        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), Some(&2));
        assert_eq!(cache.get(&"key3".to_string()), Some(&3));
    }

    #[test]
    fn test_lru_cache_update_order() {
        let mut cache = LruCacheImpl::<String, i32>::new(2);

        cache.add("key1".to_string(), 1);
        cache.add("key2".to_string(), 2);

        // Access key1, making key2 the least recently used
        cache.get(&"key1".to_string());

        // Add key3, should evict key2
        cache.add("key3".to_string(), 3);

        assert_eq!(cache.get(&"key1".to_string()), Some(&1));
        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.get(&"key3".to_string()), Some(&3));
    }
}
