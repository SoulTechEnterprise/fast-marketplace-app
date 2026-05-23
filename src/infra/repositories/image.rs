use std::{env, fs, path::PathBuf};

use reqwest::Client;
use uuid::Uuid;

use async_trait::async_trait;

use image::ImageFormat;
use std::io::Cursor;

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
        const MAX_SIZE: usize = 4 * 1024 * 1024;

        let tasks: Vec<_> = urls
            .into_iter()
            .map(|url| {
                let client = self.client.clone();
                let storage_dir = self.storage_dir.clone();

                tokio::spawn(async move {
                    let response = client.get(&url).send().await.ok()?;
                    let bytes = response.bytes().await.ok()?;

                    if bytes.len() > MAX_SIZE {
                        return None;
                    }

                    let img = image::load_from_memory(&bytes).ok()?;

                    let jpg_bytes = tokio::task::spawn_blocking(move || {
                        let mut buf = Cursor::new(Vec::new());
                        img.write_to(&mut buf, ImageFormat::Jpeg).ok()?;
                        Some(buf.into_inner())
                    })
                    .await
                    .ok()??;

                    let filename = format!("{}.jpg", Uuid::new_v4());
                    let local_path = storage_dir.join(&filename);
                    tokio::fs::write(&local_path, &jpg_bytes).await.ok()?;
                    let canon = local_path.canonicalize().ok()?;
                    canon.to_str().map(|s| s.to_string())
                })
            })
            .collect();

        futures::future::join_all(tasks)
            .await
            .into_iter()
            .filter_map(|r| r.ok().flatten())
            .collect()
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
