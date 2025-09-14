use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A simple time-based cache for expensive operations
#[derive(Debug)]
pub struct Cache<T> {
    data: HashMap<String, CacheEntry<T>>,
    default_ttl: Duration,
}

#[derive(Debug)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> Cache<T> {
    /// Create a new cache with default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            default_ttl,
        }
    }

    /// Get a value from cache if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<&T> {
        if let Some(entry) = self.data.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(&entry.value);
            }
        }
        None
    }

    /// Insert a value into cache with default TTL
    pub fn insert(&mut self, key: String, value: T) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Insert a value into cache with custom TTL
    pub fn insert_with_ttl(&mut self, key: String, value: T, ttl: Duration) {
        let expires_at = Instant::now() + ttl;
        self.data.insert(key, CacheEntry { value, expires_at });
    }

    /// Remove expired entries from cache
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, entry| now < entry.expires_at);
    }

    /// Clear all entries from cache
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the number of entries in cache (including expired ones)
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T> Default for Cache<T> {
    fn default() -> Self {
        // Default 30 second TTL
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = Cache::new(Duration::from_secs(1));
        cache.insert("test".to_string(), 42);

        assert_eq!(cache.get("test"), Some(&42));
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = Cache::new(Duration::from_millis(50));
        cache.insert("test".to_string(), 42);

        // Should still be valid
        assert_eq!(cache.get("test"), Some(&42));

        // Wait for expiration
        thread::sleep(Duration::from_millis(60));

        // Should be expired
        assert_eq!(cache.get("test"), None);
    }

    #[test]
    fn test_cache_custom_ttl() {
        let mut cache = Cache::new(Duration::from_secs(1)); // default 1s
        cache.insert_with_ttl("short".to_string(), 1, Duration::from_millis(50));
        cache.insert_with_ttl("long".to_string(), 2, Duration::from_secs(2));

        // Both should be valid initially
        assert_eq!(cache.get("short"), Some(&1));
        assert_eq!(cache.get("long"), Some(&2));

        // Wait for short to expire but long to remain
        thread::sleep(Duration::from_millis(60));

        assert_eq!(cache.get("short"), None);
        assert_eq!(cache.get("long"), Some(&2));
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = Cache::new(Duration::from_millis(50));
        cache.insert("test1".to_string(), 1);
        cache.insert("test2".to_string(), 2);

        // Wait for expiration
        thread::sleep(Duration::from_millis(60));

        // Should be expired
        assert_eq!(cache.get("test1"), None);
        assert_eq!(cache.get("test2"), None);

        // But still in data structure
        assert_eq!(cache.len(), 2);

        // Cleanup should remove expired entries
        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }
}
