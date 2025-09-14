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
    fn test_cache_mixed_expiration_cleanup() {
        let mut cache = Cache::new(Duration::from_secs(1));

        // Insert entries with different TTLs
        cache.insert_with_ttl("short".to_string(), 1, Duration::from_millis(50));
        cache.insert_with_ttl("long".to_string(), 2, Duration::from_secs(2));

        // Wait for short to expire
        thread::sleep(Duration::from_millis(60));

        // Short should be expired, long should still be valid
        assert_eq!(cache.get("short"), None);
        assert_eq!(cache.get("long"), Some(&2));

        // Should have 2 entries (1 expired, 1 valid)
        assert_eq!(cache.len(), 2);

        // Cleanup should only remove expired entries
        cache.cleanup();
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("long"), Some(&2));
        assert_eq!(cache.get("short"), None);
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
    fn test_cache_operations_after_cleanup() {
        let mut cache = Cache::new(Duration::from_millis(50));

        // Add and expire some entries
        cache.insert("expired1".to_string(), 1);
        cache.insert("expired2".to_string(), 2);
        thread::sleep(Duration::from_millis(60));

        // Add a fresh entry
        cache.insert("fresh".to_string(), 3);

        // Cleanup expired entries
        cache.cleanup();

        // Should only have the fresh entry
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("fresh"), Some(&3));
        assert_eq!(cache.get("expired1"), None);
        assert_eq!(cache.get("expired2"), None);

        // Should be able to add more entries
        cache.insert("new".to_string(), 4);
        assert_eq!(cache.get("new"), Some(&4));
    }

    #[test]
    fn test_cache_empty_operations() {
        let mut cache: Cache<i32> = Cache::new(Duration::from_secs(1));

        // Empty cache operations
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get("anything"), None);

        // Clear empty cache should work
        cache.clear();
        assert!(cache.is_empty());

        // Cleanup empty cache should work
        cache.cleanup();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_large_number_of_entries() {
        let mut cache = Cache::new(Duration::from_secs(10));

        // Add many entries
        for i in 0..1000 {
            cache.insert(format!("key_{}", i), i);
        }

        assert_eq!(cache.len(), 1000);
        assert!(!cache.is_empty());

        // Verify some entries
        assert_eq!(cache.get("key_0"), Some(&0));
        assert_eq!(cache.get("key_500"), Some(&500));
        assert_eq!(cache.get("key_999"), Some(&999));

        // Clear should work
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_default_ttl() {
        let cache: Cache<i32> = Cache::default();

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
