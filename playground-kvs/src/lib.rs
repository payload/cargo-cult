#![deny(missing_docs)]

//! kvs contains a key-value store implementation.

use std::collections::BTreeMap;

/// A key-value which works but gives no guarantees about complexity, performance, edge-cases.
/// Keys and values will be copied.
/// Operations on non existent keys will be silently ignored.
/// Keys are compared by Eq.
/// That means that different keys, which are equal by Eq or change their equalness
/// over time, might produce unintended behaviour.
pub struct KvStore {
    store: BTreeMap<String, String>,
}

impl KvStore {
    /// Returns a new empty store.
    /// Getting a value by a key will return None.
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }

    /// Inserts a new key-value entry or overwrites an existing one with an equal key.
    /// You can remove that entry or get its value after that call.
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// If an equal key was inserted before and not removed yet,
    /// then a copy of its value is returned.
    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).cloned()
    }

    /// Removes one entry with an equal key.
    /// You can't get its value after that call.
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
