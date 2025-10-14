//! File access commands for Tauri frontend
//!
//! Provides commands to safely read files and convert them to formats
//! that can be used by the frontend, since Tauri doesn't allow direct
//! file:// protocol access for security reasons.

use std::path::Path;

/// Converts a file path to an asset URL that can be used by the frontend.
///
/// This uses Tauri's convertFileSrc which creates a URL that the asset protocol
/// can handle. The asset protocol supports range requests, making it ideal for
/// streaming large video files.
///
/// # Arguments
///
/// * `file_path` - The absolute path to the file to convert
///
/// # Returns
///
/// Returns `Ok(String)` with the asset URL, or `Err(String)` on failure.
///
/// # Errors
///
/// Returns an error if:
/// * The file does not exist
/// * The path is invalid
#[tauri::command]
pub fn get_asset_url(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }
    
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", file_path));
    }
    
    // Return the path - the frontend will use convertFileSrc on it
    Ok(file_path)
}

/// Gets the MIME type for a file based on its extension.
///
/// # Arguments
///
/// * `file_path` - The file path to determine the MIME type for
///
/// # Returns
///
/// Returns the MIME type string, or "application/octet-stream" if unknown.
#[tauri::command]
pub fn get_mime_type(file_path: String) -> String {
    let path = Path::new(&file_path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "heic" | "heif" => "image/heic",
        "tiff" | "tif" => "image/tiff",
        
        // Videos
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "flv" => "video/x-flv",
        "wmv" => "video/x-ms-wmv",
        "m4v" => "video/x-m4v",
        "mpg" | "mpeg" => "video/mpeg",
        
        _ => "application/octet-stream",
    }
    .to_string()
}
