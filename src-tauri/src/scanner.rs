use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Represents a media file with its metadata.
///
/// Contains all relevant information about a discovered media file including
/// its location, size, modification time, and type classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaFile {
    /// Absolute path to the file
    pub path: String,
    /// File name without path
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp (Unix timestamp)
    pub modified: i64,
    /// File type/extension (e.g., "jpg", "mp4")
    pub file_type: String,
    /// Whether it's an image or video
    pub media_type: MediaType,
}

/// Classification of media file types.
///
/// Used to distinguish between different types of media files
/// that the application can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Image file (jpg, png, etc.)
    Image,
    /// Video file (mp4, mov, etc.)
    Video,
    /// Unsupported or unknown file type
    Unknown,
}

impl Default for MediaType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Supported image extensions
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "heic", "heif", "tiff", "tif", "svg",
];

/// Supported video extensions
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "webm", "flv", "wmv", "m4v", "mpg", "mpeg",
];

/// Determines the media type based on file extension.
///
/// # Arguments
///
/// * `extension` - The file extension (without the dot) in lowercase
///
/// # Returns
///
/// Returns the corresponding `MediaType` for the extension.
fn determine_media_type(extension: &str) -> MediaType {
    if IMAGE_EXTENSIONS.contains(&extension) {
        MediaType::Image
    } else if VIDEO_EXTENSIONS.contains(&extension) {
        MediaType::Video
    } else {
        MediaType::Unknown
    }
}

/// Scans a directory for media files.
///
/// Recursively or non-recursively scans the specified directory path for
/// supported image and video files, collecting metadata for each discovered file.
///
/// # Arguments
///
/// * `path` - The directory path to scan (as a String for Tauri command compatibility)
/// * `recursive` - Whether to scan subdirectories recursively
///
/// # Returns
///
/// Returns `Ok(Vec<MediaFile>)` containing all discovered media files sorted by
/// modification date (newest first), or `Err(String)` if the path is invalid.
///
/// # Errors
///
/// Returns an error if:
/// * The path does not exist
/// * The path is not a directory
///
/// # Examples
///
/// ```no_run
/// # use fmlm_lib::scan_directory;
/// let files = scan_directory("/path/to/photos".to_string(), true)?;
/// # Ok::<(), String>(())
/// ```
#[tauri::command]
pub fn scan_directory(path: String, recursive: bool) -> Result<Vec<MediaFile>, String> {
    let path = Path::new(&path);
    
    // Validate path exists
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    // Validate path is a directory
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }
    
    let mut media_files = Vec::new();
    
    let walker = if recursive {
        WalkDir::new(path).follow_links(true)
    } else {
        WalkDir::new(path).max_depth(1).follow_links(true)
    };
    
    for entry in walker.into_iter().filter_map(Result::ok) {
        // Skip directories, only process files
        if entry.file_type().is_dir() {
            continue;
        }
        
        let file_path = entry.path();
        
        // Extract and normalize file extension
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();
        
        // Determine media type based on extension
        let media_type = determine_media_type(&extension);
        
        // Skip unsupported file types
        if media_type == MediaType::Unknown {
            continue;
        }
        
        // Extract file metadata
        if let Ok(metadata) = entry.metadata() {
            let modified = extract_modified_timestamp(&metadata);
            let file_name = extract_file_name(file_path);
            
            media_files.push(MediaFile {
                path: file_path.to_string_lossy().to_string(),
                name: file_name,
                size: metadata.len(),
                modified,
                file_type: extension,
                media_type,
            });
        }
    }
    
    // Sort by modification date (newest first)
    media_files.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    Ok(media_files)
}

/// Extracts the Unix timestamp from file metadata.
///
/// # Arguments
///
/// * `metadata` - The file metadata
///
/// # Returns
///
/// Returns the Unix timestamp as i64, or 0 if unavailable.
fn extract_modified_timestamp(metadata: &std::fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
        })
        .unwrap_or(0)
}

/// Extracts the file name from a path.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// Returns the file name as a String, or "Unknown" if it cannot be extracted.
fn extract_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_determine_media_type_image() {
        assert_eq!(determine_media_type("jpg"), MediaType::Image);
        assert_eq!(determine_media_type("png"), MediaType::Image);
        assert_eq!(determine_media_type("gif"), MediaType::Image);
        assert_eq!(determine_media_type("webp"), MediaType::Image);
    }
    
    #[test]
    fn test_determine_media_type_video() {
        assert_eq!(determine_media_type("mp4"), MediaType::Video);
        assert_eq!(determine_media_type("mov"), MediaType::Video);
        assert_eq!(determine_media_type("avi"), MediaType::Video);
        assert_eq!(determine_media_type("mkv"), MediaType::Video);
    }
    
    #[test]
    fn test_determine_media_type_unknown() {
        assert_eq!(determine_media_type("txt"), MediaType::Unknown);
        assert_eq!(determine_media_type("pdf"), MediaType::Unknown);
        assert_eq!(determine_media_type(""), MediaType::Unknown);
    }
    
    #[test]
    fn test_media_type_default() {
        assert_eq!(MediaType::default(), MediaType::Unknown);
    }
    
    #[test]
    fn test_media_file_equality() {
        let file1 = MediaFile {
            path: "/test/file.jpg".to_string(),
            name: "file.jpg".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: "jpg".to_string(),
            media_type: MediaType::Image,
        };
        
        let file2 = file1.clone();
        assert_eq!(file1, file2);
    }
    
    #[test]
    fn test_scan_directory_invalid_path() {
        let result = scan_directory("/nonexistent/path".to_string(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }
}
