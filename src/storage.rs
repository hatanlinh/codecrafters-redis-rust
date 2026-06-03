use std::collections::HashMap;

pub struct Storage {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &Vec<u8>) -> Option<&Vec<u8>> {
        self.data.get(key)
    }
}
