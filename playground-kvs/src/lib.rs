use std::collections::BTreeMap;

pub struct KvStore {
    store: BTreeMap<String, String>
}

impl KvStore {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).cloned()
    }

    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
