use rpds::{HashTrieMap, Vector}; // Import persistent map and vector

#[derive(Debug, Clone)]
pub struct ReorderableMap<K: Eq + std::hash::Hash + Clone, V: Clone> {
    map: HashTrieMap<K, V>, // Persistent map for key-value storage
    order: Vector<K>,       // Persistent vector for maintaining insertion order
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> ReorderableMap<K, V> {
    // Create a new empty reorderable map
    pub fn new() -> Self {
        ReorderableMap {
            map: HashTrieMap::new(),
            order: Vector::new(),
        }
    }

    pub fn index_of_key(&self, key: &K) -> Option<usize> {
        self.order.iter().position(|k| k == key)
    }

    // Insert a new key-value pair, returning a new instance of ReorderableMap
    pub fn insert(&self, key: K, value: V) -> Self {
        let new_order = if !self.map.contains_key(&key) {
            self.order.push_back(key.clone())
        } else {
            self.order.clone()
        };

        let new_map = self.map.insert(key, value);

        ReorderableMap {
            map: new_map,
            order: new_order,
        }
    }

    // Get value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    // Get value by index without cloning
    pub fn get_by_index(&self, index: usize) -> Option<(&K, &V)> {
        self.order
            .get(index)
            .and_then(|key| self.map.get(key).map(|value| (key, value)))
    }

    // Swap the order of two elements by index, returning a new ReorderableMap
    pub fn swap(&self, i: usize, j: usize) -> Self {
        if i < self.order.len() && j < self.order.len() {
            let key_i = self.order.get(i).unwrap().clone();
            let key_j = self.order.get(j).unwrap().clone();
            let new_order = self
                .order
                .set(i, key_j)
                .expect("Index out of bounds")
                .set(j, key_i)
                .expect("Index out of bounds");

            ReorderableMap {
                map: self.map.clone(),
                order: new_order,
            }
        } else {
            self.clone()
        }
    }

    // Get all values in the current order
    pub fn ordered_values(&self) -> Vec<&V> {
        self.order
            .iter()
            .filter_map(|key| self.map.get(key))
            .collect()
    }

    // Get all keys in the current order
    pub fn ordered_keys(&self) -> Vec<&K> {
        self.order.iter().collect()
    }

    // Set a new order by passing a new vector of keys, returning a new instance
    pub fn set_order(&self, new_order: Vector<K>) -> Self {
        if new_order.len() == self.order.len() && new_order.iter().all(|k| self.map.contains_key(k))
        {
            ReorderableMap {
                map: self.map.clone(),
                order: new_order,
            }
        } else {
            self.clone()
        }
    }

    // Remove a key-value pair by key, returning a new instance of ReorderableMap
    pub fn remove(&self, key: &K) -> Option<Self> {
        if let Some(_) = self.map.get(key) {
            let new_map = self.map.remove(key);
            let new_order = self.order.iter().filter(|k| *k != key).cloned().collect();

            Some(ReorderableMap {
                map: new_map,
                order: new_order,
            })
        } else {
            None
        }
    }

    // Remove a key-value pair by index, returning a new ReorderableMap and the removed value
    pub fn remove_by_index(&self, index: usize) -> Option<(Self, (K, V))> {
        let key = self.order.get(index)?.clone();
        let value = self.map.get(&key)?.clone();

        let new_map = self.map.remove(&key);
        let new_order = self
            .order
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, k)| k.clone())
            .collect();

        let new_reorderable_map = ReorderableMap {
            map: new_map,
            order: new_order,
        };

        Some((new_reorderable_map, (key, value)))
    }

    // Get the length of the map
    pub fn len(&self) -> usize {
        self.order.len()
    }

    // Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    // Provides an iterator over (&K, &V) without consuming the map
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.order
            .iter()
            .filter_map(move |key| self.map.get(key).map(|value| (key, value)))
    }

    // Implement values() to return an iterator over the values in the current order
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.order.iter().filter_map(move |key| self.map.get(key))
    }
}

// IntoIterator for owned ReorderableMap (consumes the map)
impl<K: Clone + Eq + std::hash::Hash, V: Clone> IntoIterator for ReorderableMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.order
            .iter()
            .filter_map(|key| self.map.get(key).map(|value| (key.clone(), value.clone())))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

// IntoIterator for &ReorderableMap (borrows the map)
impl<'a, K: Clone + Eq + std::hash::Hash, V: Clone> IntoIterator for &'a ReorderableMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::vec::IntoIter<(&'a K, &'a V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.order
            .iter()
            .filter_map(|key| self.map.get(key).map(|value| (key, value)))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> FromIterator<(K, V)> for ReorderableMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = ReorderableMap::new();
        for (key, value) in iter {
            map = map.insert(key, value); // Since insert returns a new map, reassign it
        }
        map
    }
}
