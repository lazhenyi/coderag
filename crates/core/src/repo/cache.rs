//! LRU Cache implementation for git objects

use linked_hash_map::LinkedHashMap;

/// A simple LRU cache with bounded capacity
pub struct LruCache<K: std::hash::Hash + Eq + Clone, V> {
    map: LinkedHashMap<K, V>,
    capacity: usize,
}

impl<K: std::hash::Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: LinkedHashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Move to end (most recently used)
        if self.map.contains_key(key) {
            let val = self.map.remove(key).unwrap();
            self.map.insert(key.clone(), val);
            self.map.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, val: V) {
        // Remove if exists to update position
        if self.map.contains_key(&key) {
            self.map.remove(&key);
        }

        // Insert at end (most recently used)
        self.map.insert(key, val);

        // Evict oldest if over capacity
        while self.map.len() > self.capacity {
            if let Some(oldest) = self.map.pop_front() {
                let _ = oldest; // Drop the oldest entry
            }
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(3);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));

        // Adding d should evict the oldest (b, since a was accessed)
        cache.put("d", 4);

        assert!(!cache.map.contains_key(&"b"));
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"c"), Some(&3));
        assert_eq!(cache.get(&"d"), Some(&4));
    }
}
