pub struct KvStore {}

impl KvStore {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set(&mut self, _key: String, _value: String) {
        unimplemented!();
    }

    pub fn get(&self, _key: String) -> Option<String> {
        unimplemented!();
    }

    pub fn remove(&mut self, _key: String) {
        unimplemented!();
    }
}
