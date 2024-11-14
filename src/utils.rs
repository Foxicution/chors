use rpds::{HashTrieMap, Vector};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[macro_export]
macro_rules! persistent_map {
    ( $( $key:expr => $value:expr ),* ) => {{
        let map = PersistentIndexMap::new();
        $(
            let map = map.insert($key, $value);
        )*
        map
    }};
}

// Define an extension trait with the `to_vec` method
pub trait VectorUtils<T> {
    fn to_vec(&self) -> Vec<T>;
}

// Implement the trait for `Vector<T>`
impl<T: Clone> VectorUtils<T> for Vector<T> {
    fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentIndexMap<K: Eq + Hash, V> {
    // Add `Eq` and `Hash` constraints
    map: HashTrieMap<K, V>, // Persistent map for key-value storage
    order: Vector<K>,       // Persistent vector to track insertion order
}

impl<K: Eq + Hash + Clone, V: Clone> PersistentIndexMap<K, V> {
    // Create a new empty PersistentIndexMap
    pub fn new() -> Self {
        Self {
            map: HashTrieMap::new(),
            order: Vector::new(),
        }
    }

    // Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Self {
        let mut new_order = self.order.clone();
        if !self.map.contains_key(&key) {
            new_order = new_order.push_back(key.clone()); // Append key to order
        }
        Self {
            map: self.map.insert(key, value),
            order: new_order,
        }
    }

    // Remove a key-value pair and preserve relative order
    pub fn remove(&self, key: &K) -> Self {
        if self.map.contains_key(key) {
            let new_order: Vector<K> = self.order.iter().filter(|&k| k != key).cloned().collect();
            Self {
                map: self.map.remove(key),
                order: new_order,
            }
        } else {
            self.clone() // Return current state if key doesn't exist
        }
    }

    // Swap the positions of two keys in the insertion order
    pub fn swap(&self, key1: &K, key2: &K) -> Option<Self> {
        let idx1 = self.order.iter().position(|k| k == key1)?;
        let idx2 = self.order.iter().position(|k| k == key2)?;
        let mut new_order = self.order.clone();

        // Handle the Option returned by `set` by using `?` to propagate `None` if an index is invalid
        new_order = new_order.set(idx1, key2.clone())?;
        new_order = new_order.set(idx2, key1.clone())?;

        Some(Self {
            map: self.map.clone(),
            order: new_order,
        })
    }

    // Get a reference to the value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_index(&self, key: &K) -> Option<usize> {
        self.order.iter().position(|k| k == key)
    }

    // Get a key at index
    pub fn get_key_at_index(&self, index: usize) -> Option<&K> {
        self.order.get(index)
    }

    // Check if a key exists in the map
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    // Get an iterator over the key-value pairs in insertion order
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.order
            .iter()
            .filter_map(move |key| self.map.get(key).map(|value| (key, value)))
    }

    // Get the length of the map
    pub fn len(&self) -> usize {
        self.map.size()
    }

    // Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    // Clear all elements in the map
    pub fn clear(&self) -> Self {
        Self {
            map: HashTrieMap::new(),
            order: Vector::new(),
        }
    }

    pub fn keys(&self) -> &Vector<K> {
        &self.order
    }

    pub fn keys_to_vec(&self) -> Vec<K> {
        self.order.to_vec()
    }

    pub fn values(&self) -> Vector<V> {
        self.order
            .iter()
            .filter_map(|key| self.map.get(key))
            .cloned()
            .collect()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> FromIterator<(K, V)> for PersistentIndexMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = HashTrieMap::new();
        let mut order = Vector::new();

        for (key, value) in iter {
            map = map.insert(key.clone(), value);
            order = order.push_back(key);
        }

        Self { map, order }
    }
}

impl<K: Eq + Hash + Clone, V: PartialEq> PartialEq for PersistentIndexMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map && self.order == other.order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);

        // Ensure that the values are inserted correctly
        assert_eq!(map.get(&"key1".to_string()), Some(&10));
        assert_eq!(map.get(&"key2".to_string()), Some(&20));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_insert_existing_key() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key1".to_string(), 30); // Overwriting "key1"

        // Ensure that the value is updated and not duplicated in the order
        assert_eq!(map.get(&"key1".to_string()), Some(&30));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_remove() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);
        let map = map.remove(&"key1".to_string());

        // Ensure that the key is removed
        assert_eq!(map.get(&"key1".to_string()), None);
        assert_eq!(map.get(&"key2".to_string()), Some(&20));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_remove_nonexistent_key() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.remove(&"key2".to_string()); // Removing a key that doesn't exist

        // Ensure that nothing changes when removing a nonexistent key
        assert_eq!(map.get(&"key1".to_string()), Some(&10));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_swap() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);
        let map = map.insert("key3".to_string(), 30);

        if let Some(map) = map.swap(&"key1".to_string(), &"key3".to_string()) {
            let mut iter = map.iter();

            // After swapping, key3 should come first, followed by key2, then key1
            assert_eq!(iter.next(), Some((&"key3".to_string(), &30)));
            assert_eq!(iter.next(), Some((&"key2".to_string(), &20)));
            assert_eq!(iter.next(), Some((&"key1".to_string(), &10)));
        } else {
            panic!("Swap failed");
        }
    }

    #[test]
    fn test_swap_nonexistent_keys() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);

        // Attempt to swap nonexistent keys
        assert!(map.swap(&"key1".to_string(), &"key2".to_string()).is_none());
    }

    #[test]
    fn test_iter_in_order() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);
        let map = map.insert("key3".to_string(), 30);

        let mut iter = map.iter();

        // Ensure iteration is in insertion order
        assert_eq!(iter.next(), Some((&"key1".to_string(), &10)));
        assert_eq!(iter.next(), Some((&"key2".to_string(), &20)));
        assert_eq!(iter.next(), Some((&"key3".to_string(), &30)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_clear() {
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);
        let map = map.clear();

        // Ensure that the map is empty after clear
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_order_preservation_after_removal() {
        // Test that order is preserved after removing an element
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);
        let map = map.insert("key3".to_string(), 30);

        // Remove key2
        let map = map.remove(&"key2".to_string());

        // The order should now be key1, key3
        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((&"key1".to_string(), &10)));
        assert_eq!(iter.next(), Some((&"key3".to_string(), &30)));
    }

    #[test]
    fn test_get_key_at_index_out_of_bounds() {
        // Test that getting a key at an out-of-bounds index returns None
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);

        // Index out of bounds
        assert!(map.get_key_at_index(1).is_none());
    }

    #[test]
    fn test_clear_preserves_structure() {
        // Test that clear resets the map correctly
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.clear();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(map.get(&"key1".to_string()).is_none());
    }

    #[test]
    fn test_get_index() {
        // Test get_index method
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);

        assert_eq!(map.get_index(&"key1".to_string()), Some(0));
        assert_eq!(map.get_index(&"key2".to_string()), Some(1));
        assert_eq!(map.get_index(&"key3".to_string()), None);
    }

    #[test]
    fn test_contains_key() {
        // Test contains_key method
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);

        assert!(map.contains_key(&"key1".to_string()));
        assert!(!map.contains_key(&"key2".to_string()));
    }

    #[test]
    fn test_keys() {
        // Test keys method
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);

        let keys = map.keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys.get(0), Some(&"key1".to_string()));
        assert_eq!(keys.get(1), Some(&"key2".to_string()));
    }

    #[test]
    fn test_keys_to_vec() {
        // Test keys_to_vec method
        let map = PersistentIndexMap::new();
        let map = map.insert("key1".to_string(), 10);
        let map = map.insert("key2".to_string(), 20);

        let keys_vec = map.keys_to_vec();
        assert_eq!(keys_vec.len(), 2);
        assert_eq!(keys_vec[0], "key1".to_string());
        assert_eq!(keys_vec[1], "key2".to_string());
    }

    #[test]
    fn test_from_iter() {
        // Test FromIterator implementation
        let entries = vec![
            ("key1".to_string(), 10),
            ("key2".to_string(), 20),
            ("key3".to_string(), 30),
        ];
        let map: PersistentIndexMap<String, i32> = entries.into_iter().collect();

        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&"key1".to_string()), Some(&10));
        assert_eq!(map.get(&"key2".to_string()), Some(&20));
        assert_eq!(map.get(&"key3".to_string()), Some(&30));

        let keys_vec = map.keys_to_vec();
        assert_eq!(keys_vec[0], "key1".to_string());
        assert_eq!(keys_vec[1], "key2".to_string());
        assert_eq!(keys_vec[2], "key3".to_string());
    }
}
