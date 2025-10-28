//! Tauri commands for thumbnail operations

use crate::thumbnail;
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};
use tokio::sync::Semaphore;
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::fs;
use base64::{Engine as _, engine::general_purpose};

// Global semaphore to limit concurrent thumbnail generation
static THUMBNAIL_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(5)));

/// Convert a thumbnail file to a base64 data URL
fn thumbnail_to_data_url(thumbnail_path: &str) -> Result<String, String> {
    let data = fs::read(thumbnail_path)
        .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;
    
    let base64_data = general_purpose::STANDARD.encode(&data);
    Ok(format!("data:image/jpeg;base64,{}", base64_data))
}

/// Response for thumbnail generation request
#[derive(Debug, Serialize, Deserialize)]
pub struct ThumbnailResponse {
    pub success: bool,
    pub thumbnail_path: Option<String>,
    pub thumbnail_data_url: Option<String>,
    pub error: Option<String>,
}

/// Generate a thumbnail for a media file
///
/// # Arguments
///
/// * `file_path` - Path to the media file
/// * `is_video` - Whether the file is a video
///
/// # Returns
///
/// ThumbnailResponse with the path to the thumbnail or error
#[tauri::command]
pub async fn generate_thumbnail(file_path: String, is_video: bool) -> ThumbnailResponse {
    info!("Thumbnail generation requested for: {}", file_path);
    
    // Acquire semaphore permit to limit concurrency
    let _permit = THUMBNAIL_SEMAPHORE.acquire().await.unwrap();
    debug!("Acquired semaphore permit for thumbnail generation");
    
    match thumbnail::generate_thumbnail(&file_path, is_video) {
        Ok(thumbnail_path) => {
            info!("Thumbnail generated successfully: {}", thumbnail_path);
            
            // Convert to data URL for browser compatibility
            let data_url = thumbnail_to_data_url(&thumbnail_path).ok();
            
            ThumbnailResponse {
                success: true,
                thumbnail_path: Some(thumbnail_path),
                thumbnail_data_url: data_url,
                error: None,
            }
        },
        Err(e) => {
            error!("Thumbnail generation failed for {}: {}", file_path, e);
            ThumbnailResponse {
                success: false,
                thumbnail_path: None,
                thumbnail_data_url: None,
                error: Some(e.to_string()),
            }
        },
    }
}

/// Check if a thumbnail exists for a file
///
/// # Arguments
///
/// * `file_path` - Path to the media file
///
/// # Returns
///
/// Boolean indicating if thumbnail exists
#[tauri::command]
pub fn thumbnail_exists(file_path: String) -> bool {
    thumbnail::thumbnail_exists(&file_path).unwrap_or(false)
}

/// Get the path to a thumbnail if it exists
///
/// # Arguments
///
/// * `file_path` - Path to the media file
///
/// # Returns
///
/// Option with thumbnail path if it exists
#[tauri::command]
pub fn get_thumbnail_path(file_path: String) -> Option<String> {
    match thumbnail::get_thumbnail_path(&file_path) {
        Ok(path) => {
            if path.exists() {
                // Return data URL instead of file path
                thumbnail_to_data_url(&path.to_string_lossy()).ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Clear all cached thumbnails
///
/// # Returns
///
/// Result indicating success or failure
#[tauri::command]
pub fn clear_thumbnail_cache() -> Result<(), String> {
    thumbnail::clear_cache().map_err(|e| e.to_string())
}

/// Get the size of the thumbnail cache
///
/// # Returns
///
/// Cache size in bytes
#[tauri::command]
pub fn get_cache_size() -> Result<u64, String> {
    thumbnail::get_cache_size().map_err(|e| e.to_string())
}

/// Batch generate thumbnails for multiple files
///
/// # Arguments
///
/// * `files` - Vector of (file_path, is_video) tuples
///
/// # Returns
///
/// Vector of ThumbnailResponse for each file
#[tauri::command]
pub async fn generate_thumbnails_batch(files: Vec<(String, bool)>) -> Vec<ThumbnailResponse> {
    info!("Batch thumbnail generation requested for {} files", files.len());
    use futures::stream::{self, StreamExt};
    
    // Process files in parallel with concurrency limit
    let results: Vec<ThumbnailResponse> = stream::iter(files)
        .map(|(file_path, is_video)| async move {
            // Acquire semaphore permit
            let _permit = THUMBNAIL_SEMAPHORE.acquire().await.unwrap();
            debug!("Processing thumbnail for: {}", file_path);
            
            match thumbnail::generate_thumbnail(&file_path, is_video) {
                Ok(thumbnail_path) => {
                    debug!("Batch thumbnail generated: {}", thumbnail_path);
                    
                    // Convert to data URL for browser compatibility
                    let data_url = thumbnail_to_data_url(&thumbnail_path).ok();
                    
                    ThumbnailResponse {
                        success: true,
                        thumbnail_path: Some(thumbnail_path),
                        thumbnail_data_url: data_url,
                        error: None,
                    }
                },
                Err(e) => {
                    error!("Batch thumbnail failed for {}: {}", file_path, e);
                    ThumbnailResponse {
                        success: false,
                        thumbnail_path: None,
                        thumbnail_data_url: None,
                        error: Some(e.to_string()),
                    }
                },
            }
        })
        .buffer_unordered(5)  // Process up to 5 at a time
        .collect()
        .await;
    
    info!("Batch thumbnail generation completed: {} files processed", results.len());
    results
}
