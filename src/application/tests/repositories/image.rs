use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::repositories::image::ImageRepository;

pub struct InMemoryImageRepository {
    storage: Mutex<Vec<String>>,
}

impl InMemoryImageRepository {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryImageRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImageRepository for InMemoryImageRepository {
    async fn add(&self, body: Vec<String>) -> Vec<String> {
        let mut storage = self.storage.lock().unwrap();
        storage.extend(body.clone());
        body
    }

    async fn remove(&self) -> () {
        let mut storage = self.storage.lock().unwrap();
        storage.clear();
    }
}
