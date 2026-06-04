use std::{env, fs, path::PathBuf, sync::Arc};

use reqwest::Client;
use tokio::sync::Semaphore;
use uuid::Uuid;

use async_trait::async_trait;

use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

use crate::domain::repositories::image::ImageRepository;
use crate::infra::logger;

// ─── Constantes de configuração ─────────────────────────────────────────────

/// Tamanho máximo permitido para uma imagem (10 MB).
const MAX_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Número máximo de imagens por requisição.
const MAX_IMAGES_PER_REQUEST: usize = 20;

/// Número máximo de downloads simultâneos.
const MAX_CONCURRENT_DOWNLOADS: usize = 5;

/// Qualidade JPEG inicial para compressão.
const JPEG_QUALITY_START: u8 = 90;

/// Qualidade JPEG mínima (para não ficar feio).
const JPEG_QUALITY_MIN: u8 = 40;

/// Passo de redução de qualidade JPEG por tentativa.
const JPEG_QUALITY_STEP: u8 = 10;

/// Escalas de redimensionamento progressivo (em porcentagem).
const RESIZE_SCALES: [u32; 3] = [80, 60, 50];

// ─── Implementação do repositório ───────────────────────────────────────────

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

// ─── Otimização de imagens ──────────────────────────────────────────────────

/// Comprime uma imagem para JPEG com a qualidade especificada.
/// Retorna os bytes resultantes ou `None` se falhar.
fn compress_jpeg(img: &DynamicImage, quality: u8) -> Option<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    img.write_with_encoder(encoder).ok()?;
    Some(buf.into_inner())
}

/// Redimensiona uma imagem proporcionalmente para a escala especificada (em %).
fn resize_image(img: &DynamicImage, scale_percent: u32) -> DynamicImage {
    let new_width = (img.width() * scale_percent) / 100;
    let new_height = (img.height() * scale_percent) / 100;
    img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

/// Otimiza uma imagem que excede o tamanho máximo:
///   1. Tenta reduzir a qualidade JPEG progressivamente (90% → 40%)
///   2. Se não for suficiente, redimensiona proporcionalmente (80% → 60% → 50%)
///
/// Nunca descarta a imagem — sempre retorna algo utilizável.
fn optimize_image(img: &DynamicImage, original_size: usize) -> Vec<u8> {
    let original_mb = logger::bytes_to_mb(original_size);

    // ── Etapa 1: Tentar reduzir somente a qualidade JPEG ────────────────
    let mut quality = JPEG_QUALITY_START;
    while quality >= JPEG_QUALITY_MIN {
        if let Some(bytes) = compress_jpeg(img, quality) {
            if bytes.len() <= MAX_SIZE_BYTES {
                let new_mb = logger::bytes_to_mb(bytes.len());
                logger::image_optimized(original_mb, new_mb, quality);
                return bytes;
            }
        }
        quality = quality.saturating_sub(JPEG_QUALITY_STEP);
    }

    // ── Etapa 2: Redimensionar + comprimir na qualidade mínima ──────────
    for &scale in &RESIZE_SCALES {
        let resized = resize_image(img, scale);
        if let Some(bytes) = compress_jpeg(&resized, JPEG_QUALITY_MIN) {
            if bytes.len() <= MAX_SIZE_BYTES {
                let new_mb = logger::bytes_to_mb(bytes.len());
                logger::image_resized(original_mb, new_mb, scale);
                return bytes;
            }
        }
    }

    // ── Fallback: Retorna a melhor compressão possível ───────────────────
    // Mesmo se ainda for > 10 MB, é melhor enviar do que descartar.
    logger::warn(&format!(
        "Imagem ({:.1} MB) não pôde ser reduzida abaixo de 10 MB — enviando melhor resultado",
        original_mb
    ));
    let smallest = resize_image(img, *RESIZE_SCALES.last().unwrap());
    compress_jpeg(&smallest, JPEG_QUALITY_MIN).unwrap_or_default()
}

// ─── Trait implementation ───────────────────────────────────────────────────

#[async_trait]
impl ImageRepository for ImageRepositoryImpl {
    async fn add(&self, urls: Vec<String>) -> Vec<String> {
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS));

        // Limita número de imagens para prevenir abuso
        let urls: Vec<String> = urls.into_iter().take(MAX_IMAGES_PER_REQUEST).collect();
        let total = urls.len();

        logger::info(&format!("Processando {} imagem(ns)...", total));

        // Cria subdiretório único para esta requisição
        let storage_dir = self.create_request_dir();

        let tasks: Vec<_> = urls
            .into_iter()
            .enumerate()
            .map(|(_index, url)| {
                let client = self.client.clone();
                let storage_dir = storage_dir.clone();
                let sem = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = sem.acquire().await.ok()?;
                    let response = client.get(&url).send().await.ok()?;
                    let bytes = response.bytes().await.ok()?;
                    let raw_size = bytes.len();

                    // Carregar imagem na memória
                    let img = image::load_from_memory(&bytes).ok()?;

                    // Decidir: manter, converter ou otimizar
                    let jpg_bytes = if raw_size > MAX_SIZE_BYTES {
                        // Imagem excede 10 MB — otimizar progressivamente
                        tokio::task::spawn_blocking(move || optimize_image(&img, raw_size))
                            .await
                            .ok()?
                    } else {
                        // Imagem dentro do limite — apenas converter para JPEG
                        let size_mb = logger::bytes_to_mb(raw_size);
                        let result = tokio::task::spawn_blocking(move || {
                            let mut buf = Cursor::new(Vec::new());
                            img.write_to(&mut buf, ImageFormat::Jpeg).ok()?;
                            Some(buf.into_inner())
                        })
                        .await
                        .ok()??;

                        logger::image_kept(size_mb);
                        result
                    };

                    let filename = format!("{}.jpg", Uuid::new_v4());
                    let local_path = storage_dir.join(&filename);
                    tokio::fs::write(&local_path, &jpg_bytes).await.ok()?;
                    local_path.to_str().map(|s| s.to_string())
                })
            })
            .collect();

        let results: Vec<String> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .filter_map(|r| r.ok().flatten())
            .collect();

        let processed = results.len();
        if processed == total {
            logger::info(&format!(
                "Todas as {} imagem(ns) processadas com sucesso",
                processed
            ));
        } else {
            logger::warn(&format!(
                "{}/{} imagem(ns) processadas ({} ignoradas)",
                processed,
                total,
                total - processed
            ));
        }

        results
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
                            logger::warn(&format!(
                                "Falha ao limpar diretório de imagens {:?}: {}",
                                entry.path(),
                                e
                            ));
                        }
                    }
                }
            }
        }
    }
}
