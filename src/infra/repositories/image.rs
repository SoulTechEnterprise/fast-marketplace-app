use std::{env, fs, path::PathBuf};

use reqwest::Client;
use uuid::Uuid;

use async_trait::async_trait;

use crate::domain::repositories::image::ImageRepository;

pub struct ImageRepositoryImpl {
    client: Client,
    storage_dir: PathBuf,
}

impl ImageRepositoryImpl {
    pub fn new() -> Self {
        let storage_dir = env::temp_dir().join("webscraping_images");
        fs::create_dir_all(&storage_dir).expect("Failed to create temp image directory");

        Self {
            client: Client::new(),
            storage_dir,
        }
    }
}

impl Default for ImageRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImageRepository for ImageRepositoryImpl {
    async fn add(&self, urls: Vec<String>) -> Vec<String> {
        let mut caminhos_locais = Vec::new();

        for url in urls {
            let response = match self.client.get(&url).send().await {
                Ok(res) => res,
                Err(_) => continue,
            };

            let bytes = match response.bytes().await {
                Ok(b) => b,
                Err(_) => continue,
            };

            let ext = PathBuf::from(&url)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_string();

            let filename = format!("{}.{}", Uuid::new_v4(), ext);
            let local_path = self.storage_dir.join(&filename);

            if tokio::fs::write(&local_path, &bytes).await.is_ok() {
                if let Ok(canon) = local_path.canonicalize() {
                    if let Some(path_str) = canon.to_str() {
                        caminhos_locais.push(path_str.to_string());
                    }
                }
            }
        }

        caminhos_locais
    }

    async fn remove(&self) -> () {
        if let Ok(mut entries) = tokio::fs::read_dir(&self.storage_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
    }
}
