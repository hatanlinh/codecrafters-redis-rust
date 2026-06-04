use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Data {
    data: Vec<u8>,
    time: Instant,
    expire: Option<u64>,
}

pub struct Storage {
    storage: HashMap<Vec<u8>, Data>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>, expire_in_ms: Option<u64>) {
        self.storage.insert(
            key,
            Data {
                data: value,
                time: Instant::now(),
                expire: expire_in_ms,
            },
        );
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Option<&Vec<u8>> {
        let expired = self.storage.get(key).map_or(false, |v| {
            v.expire.map_or(false, |ex| {
                Instant::now() >= v.time + Duration::from_millis(ex)
            })
        });

        if expired {
            self.storage.remove(key);
            return None;
        }

        return self.storage.get(key).map(|v| &v.data);
    }
}
