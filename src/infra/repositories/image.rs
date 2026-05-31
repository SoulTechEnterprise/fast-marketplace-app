use std::{env, fs, path::PathBuf, sync::Arc};

use reqwest::Client;
use tokio::sync::Semaphore;
use uuid::Uuid;

use async_trait::async_trait;

use image::ImageFormat;
use std::io::Cursor;

use crate::domain::repositories::image::ImageRepository;

pub struct ImageRepositoryImpl {
    client: Client,
    base_storage_dir: PathBuf,
}

impl ImageRepositoryImpl {
    pub fn new() -> Self {
        let base_storage_dir = env::temp_dir().join("webscraping_images");
        let _ = fs::create_dir_all(&base_storage_dir);

        Self {
            client: Client::new(),
            base_storage_dir,
        }
    }

    /// Creates a unique per-request subdirectory to avoid race conditions
    /// between concurrent requests deleting each other's images.
    fn create_request_dir(&self) -> PathBuf {
        let request_id = Uuid::new_v4().to_string();
        let dir = self.base_storage_dir.join(&request_id);
        let _ = fs::create_dir_all(&dir);
        dir
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
        const MAX_IMAGES: usize = 20;
        let semaphore = Arc::new(Semaphore::new(5));

        // Limit number of images to prevent abuse
        let urls: Vec<String> = urls.into_iter().take(MAX_IMAGES).collect();

        // Create a unique subdirectory for this request
        let storage_dir = self.create_request_dir();

        let tasks: Vec<_> = urls
            .into_iter()
            .map(|url| {
                let client = self.client.clone();
                let storage_dir = storage_dir.clone();
                let sem = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = sem.acquire().await.ok()?;
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
                    local_path.to_str().map(|s| s.to_string())
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
        // Remove all per-request subdirectories that are older than 5 minutes
        // to clean up stale data from failed requests
        if let Ok(mut entries) = tokio::fs::read_dir(&self.base_storage_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_dir() {
                        // Try to remove the directory and all its contents
                        if let Err(e) = tokio::fs::remove_dir_all(entry.path()).await {
                            eprintln!(
                                "Warning: Failed to remove image directory {:?}: {}",
                                entry.path(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }
}
