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
    fn test_cache_overwrite() {
        let mut cache = Cache::new(Duration::from_secs(1));

        // Insert initial value
        cache.insert("test".to_string(), 100);
        assert_eq!(cache.get("test"), Some(&100));

        // Overwrite with new value
        cache.insert("test".to_string(), 200);
        assert_eq!(cache.get("test"), Some(&200));

        // Overwrite with custom TTL should work
        cache.insert_with_ttl("test".to_string(), 300, Duration::from_secs(2));
        assert_eq!(cache.get("test"), Some(&300));
    }



    #[test]
    fn test_cache_with_complex_types() {
        let mut cache: Cache<Vec<String>> = Cache::new(Duration::from_secs(1));

        let value1 = vec!["hello".to_string(), "world".to_string()];
        let value2 = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];

        cache.insert("list1".to_string(), value1.clone());
        cache.insert("list2".to_string(), value2.clone());

        assert_eq!(cache.get("list1"), Some(&value1));
        assert_eq!(cache.get("list2"), Some(&value2));
    }

    #[test]
    fn test_cache_very_short_ttl() {
        let mut cache = Cache::new(Duration::from_nanos(1)); // Very short TTL

        cache.insert("test".to_string(), 42);

        // Should be expired immediately (or very soon)
        thread::sleep(Duration::from_nanos(10));
        assert_eq!(cache.get("test"), None);
    }







    #[test]
    fn test_cache_default_ttl() {
        // Default should be 30 seconds
        // We can't easily test the exact TTL without accessing private fields,
        // but we can verify it works
        let mut cache = Cache::default();
        cache.insert("test".to_string(), 42);
        assert_eq!(cache.get("test"), Some(&42));
    }

    #[test]
    fn test_cache_zero_ttl() {
        let mut cache = Cache::new(Duration::from_secs(0));

        cache.insert("test".to_string(), 42);

        // Should expire immediately with 0 TTL
        assert_eq!(cache.get("test"), None);
    }

    #[test]
    fn test_cache_unicode_keys() {
        let mut cache = Cache::new(Duration::from_secs(1));

        let key = "测试_key_🚀";
        cache.insert(key.to_string(), 42);

        assert_eq!(cache.get(key), Some(&42));
    }
}
