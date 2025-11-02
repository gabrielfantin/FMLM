//! Media metadata extraction module
//!
//! Extracts detailed metadata from media files including:
//! - Resolution (width, height)
//! - Duration
//! - Codecs (video, audio)
//! - Bitrate
//! - Frame rate
//! - Additional metadata (EXIF, etc.)

use rsmpeg::avformat::AVFormatContextInput;
use rsmpeg::ffi;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, instrument};
use image::GenericImageView;
use crate::database::{self, DbPool, InsertMediaParams};
use chrono::{DateTime, Utc};
use tauri::State;

/// Error types for media info extraction
#[derive(Debug, Error)]
pub enum MediaInfoError {
    #[error("Failed to open media file: {0}")]
    FileOpen(String),
    
    #[error("Failed to read stream info: {0}")]
    StreamInfo(String),
    
    #[error("No video stream found")]
    NoVideoStream,
    
    #[error("No audio stream found")]
    NoAudioStream,
    
    #[error("Failed to extract metadata: {0}")]
    MetadataExtraction(String),
    
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
    
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

pub type MediaInfoResult<T> = Result<T, MediaInfoError>;

/// Comprehensive media file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// Video stream information
    pub video: Option<VideoInfo>,
    
    /// Audio stream information
    pub audio: Option<AudioInfo>,
    
    /// General file information
    pub general: GeneralInfo,
    
    /// Additional metadata (EXIF, tags, etc.)
    pub metadata: HashMap<String, String>,
}

/// Video stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    /// Video codec name (e.g., "h264", "hevc", "vp9")
    pub codec: String,
    
    /// Video codec long name (e.g., "H.264 / AVC / MPEG-4 AVC")
    pub codec_long: String,
    
    /// Width in pixels
    pub width: i32,
    
    /// Height in pixels
    pub height: i32,
    
    /// Frame rate (frames per second)
    pub fps: f64,
    
    /// Bitrate in bits per second (if available)
    pub bitrate: Option<i64>,
    
    /// Pixel format (e.g., "yuv420p")
    pub pix_fmt: String,
    
    /// Aspect ratio (width:height)
    pub aspect_ratio: String,
}

/// Audio stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    /// Audio codec name (e.g., "aac", "mp3", "opus")
    pub codec: String,
    
    /// Audio codec long name
    pub codec_long: String,
    
    /// Sample rate in Hz
    pub sample_rate: i32,
    
    /// Number of audio channels
    pub channels: i32,
    
    /// Bitrate in bits per second (if available)
    pub bitrate: Option<i64>,
    
    /// Sample format (e.g., "fltp")
    pub sample_fmt: String,
}

/// General file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralInfo {
    /// File format name (e.g., "mp4", "mov", "jpeg")
    pub format: String,
    
    /// Format long name
    pub format_long: String,
    
    /// Duration in seconds (for videos)
    pub duration: Option<f64>,
    
    /// Overall bitrate in bits per second
    pub bitrate: Option<i64>,
    
    /// File size in bytes
    pub size: i64,
}

/// Extract comprehensive media information from a file
#[instrument(skip_all, fields(file_path = %file_path))]
pub fn extract_media_info(file_path: &str) -> MediaInfoResult<MediaInfo> {
    debug!("Starting media info extraction");
    
    // First check if it's an image - handle separately for better EXIF support
    if is_image_file(file_path) {
        return extract_image_info(file_path);
    }
    
    // Otherwise, use FFmpeg for video/audio
    extract_video_info(file_path)
}

/// Check if file is an image based on extension
fn is_image_file(file_path: &str) -> bool {
    let extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "heic", "heif", "tiff", "tif"];
    file_path
        .rsplit('.')
        .next()
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Extract information from image files
fn extract_image_info(file_path: &str) -> MediaInfoResult<MediaInfo> {
    debug!("Extracting image info");
    
    let img = image::open(file_path)?;
    let (width, height) = img.dimensions();
    
    // Get file size
    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    
    // Determine format from extension
    let format = file_path
        .rsplit('.')
        .next()
        .unwrap_or("unknown")
        .to_uppercase();
    
    let aspect_ratio = if height > 0 {
        format!("{}:{}", width, height)
    } else {
        "N/A".to_string()
    };
    
    Ok(MediaInfo {
        video: Some(VideoInfo {
            codec: "image".to_string(),
            codec_long: format!("{} Image", format),
            width: width as i32,
            height: height as i32,
            fps: 0.0,
            bitrate: None,
            pix_fmt: format!("{:?}", img.color()),
            aspect_ratio,
        }),
        audio: None,
        general: GeneralInfo {
            format: format.clone(),
            format_long: format!("{} Image File", format),
            duration: None,
            bitrate: None,
            size: file_size,
        },
        metadata: HashMap::new(),
    })
}

/// Extract information from video files using FFmpeg
fn extract_video_info(file_path: &str) -> MediaInfoResult<MediaInfo> {
    debug!("Extracting video info with FFmpeg");
    
    let c_file_path = CString::new(file_path)
        .map_err(|_| MediaInfoError::InvalidPath("File path contains null bytes".to_string()))?;
    
    debug!("Opening media file");
    let mut input_context = AVFormatContextInput::open(&c_file_path)
        .map_err(|e| MediaInfoError::FileOpen(format!("{:?}", e)))?;
    
    debug!("Reading stream info");
    input_context.dump(0, &c_file_path)
        .map_err(|e| MediaInfoError::StreamInfo(format!("{:?}", e)))?;
    
    // Get file size
    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    
    // Extract general format info
    let format_context = input_context.as_ptr();
    let format = unsafe {
        if !(*format_context).iformat.is_null() {
            let name_ptr = (*(*format_context).iformat).name;
            if !name_ptr.is_null() {
                std::ffi::CStr::from_ptr(name_ptr)
                    .to_string_lossy()
                    .to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        }
    };
    
    let format_long = unsafe {
        if !(*format_context).iformat.is_null() {
            let long_name_ptr = (*(*format_context).iformat).long_name;
            if !long_name_ptr.is_null() {
                std::ffi::CStr::from_ptr(long_name_ptr)
                    .to_string_lossy()
                    .to_string()
            } else {
                format.clone()
            }
        } else {
            format.clone()
        }
    };
    
    let duration = unsafe {
        if (*format_context).duration != ffi::AV_NOPTS_VALUE {
            Some((*format_context).duration as f64 / ffi::AV_TIME_BASE as f64)
        } else {
            None
        }
    };
    
    let bitrate = unsafe {
        let br = (*format_context).bit_rate;
        if br > 0 {
            Some(br)
        } else {
            None
        }
    };
    
    // Extract video stream info
    let video_info = extract_video_stream_info(&mut input_context);
    
    // Extract audio stream info
    let audio_info = extract_audio_stream_info(&mut input_context);
    
    // Extract metadata
    let metadata = extract_metadata(&input_context);
    
    Ok(MediaInfo {
        video: video_info,
        audio: audio_info,
        general: GeneralInfo {
            format,
            format_long,
            duration,
            bitrate,
            size: file_size,
        },
        metadata,
    })
}

/// Extract video stream information
fn extract_video_stream_info(input_context: &mut AVFormatContextInput) -> Option<VideoInfo> {
    debug!("Looking for video stream");
    
    // Find video stream
    let video_stream_result = input_context
        .streams()
        .into_iter()
        .enumerate()
        .find(|(_, stream)| {
            unsafe {
                let codecpar = (*stream.as_ptr()).codecpar;
                if codecpar.is_null() {
                    return false;
                }
                (*codecpar).codec_type == ffi::AVMEDIA_TYPE_VIDEO
            }
        });
    
    if let Some((stream_index, stream)) = video_stream_result {
        debug!("Found video stream at index {}", stream_index);
        
        let codecpar = unsafe { (*stream.as_ptr()).codecpar };
        if codecpar.is_null() {
            return None;
        }
        
        let codec_id = unsafe { (*codecpar).codec_id };
        
        // Get codec name from descriptor
        let codec_name = unsafe {
            let codec_desc = ffi::avcodec_descriptor_get(codec_id);
            if !codec_desc.is_null() {
                std::ffi::CStr::from_ptr((*codec_desc).name)
                    .to_string_lossy()
                    .to_string()
            } else {
                "unknown".to_string()
            }
        };
        
        let codec_long_name = unsafe {
            let codec_desc = ffi::avcodec_descriptor_get(codec_id);
            if !codec_desc.is_null() {
                std::ffi::CStr::from_ptr((*codec_desc).long_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                codec_name.clone()
            }
        };
        
        let width = unsafe { (*codecpar).width };
        let height = unsafe { (*codecpar).height };
        let bitrate = unsafe {
            let br = (*codecpar).bit_rate;
            if br > 0 { Some(br) } else { None }
        };
        
        // Get frame rate
        let fps = unsafe {
            let stream_ptr = stream.as_ptr();
            let avg_frame_rate = (*stream_ptr).avg_frame_rate;
            if avg_frame_rate.den > 0 {
                avg_frame_rate.num as f64 / avg_frame_rate.den as f64
            } else {
                0.0
            }
        };
        
        // Get pixel format
        let pix_fmt = unsafe {
            let fmt = (*codecpar).format;
            let fmt_name = ffi::av_get_pix_fmt_name(fmt as i32);
            if !fmt_name.is_null() {
                std::ffi::CStr::from_ptr(fmt_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                "unknown".to_string()
            }
        };
        
        let aspect_ratio = if height > 0 {
            let gcd = gcd(width, height);
            format!("{}:{}", width / gcd, height / gcd)
        } else {
            "N/A".to_string()
        };
        
        return Some(VideoInfo {
            codec: codec_name,
            codec_long: codec_long_name,
            width,
            height,
            fps,
            bitrate,
            pix_fmt,
            aspect_ratio,
        });
    }
    
    debug!("No video stream found");
    None
}

/// Extract audio stream information
fn extract_audio_stream_info(input_context: &mut AVFormatContextInput) -> Option<AudioInfo> {
    debug!("Looking for audio stream");
    
    // Find audio stream
    let audio_stream_result = input_context
        .streams()
        .into_iter()
        .enumerate()
        .find(|(_, stream)| {
            unsafe {
                let codecpar = (*stream.as_ptr()).codecpar;
                if codecpar.is_null() {
                    return false;
                }
                (*codecpar).codec_type == ffi::AVMEDIA_TYPE_AUDIO
            }
        });
    
    if let Some((stream_index, stream)) = audio_stream_result {
        debug!("Found audio stream at index {}", stream_index);
        
        let codecpar = unsafe { (*stream.as_ptr()).codecpar };
        if codecpar.is_null() {
            return None;
        }
        
        let codec_id = unsafe { (*codecpar).codec_id };
        
        // Get codec name from descriptor
        let codec_name = unsafe {
            let codec_desc = ffi::avcodec_descriptor_get(codec_id);
            if !codec_desc.is_null() {
                std::ffi::CStr::from_ptr((*codec_desc).name)
                    .to_string_lossy()
                    .to_string()
            } else {
                "unknown".to_string()
            }
        };
        
        let codec_long_name = unsafe {
            let codec_desc = ffi::avcodec_descriptor_get(codec_id);
            if !codec_desc.is_null() {
                std::ffi::CStr::from_ptr((*codec_desc).long_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                codec_name.clone()
            }
        };
        
        let sample_rate = unsafe { (*codecpar).sample_rate };
        let channels = unsafe { (*codecpar).ch_layout.nb_channels };
        let bitrate = unsafe {
            let br = (*codecpar).bit_rate;
            if br > 0 { Some(br) } else { None }
        };
        
        // Get sample format
        let sample_fmt = unsafe {
            let fmt = (*codecpar).format;
            let fmt_name = ffi::av_get_sample_fmt_name(fmt as i32);
            if !fmt_name.is_null() {
                std::ffi::CStr::from_ptr(fmt_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                "unknown".to_string()
            }
        };
        
        return Some(AudioInfo {
            codec: codec_name,
            codec_long: codec_long_name,
            sample_rate,
            channels,
            bitrate,
            sample_fmt,
        });
    }
    
    debug!("No audio stream found");
    None
}

/// Extract metadata from the media file
fn extract_metadata(input_context: &AVFormatContextInput) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    
    unsafe {
        let format_context = input_context.as_ptr();
        let mut tag: *mut ffi::AVDictionaryEntry = std::ptr::null_mut();
        
        loop {
            tag = ffi::av_dict_get(
                (*format_context).metadata,
                std::ptr::null(),
                tag,
                ffi::AV_DICT_IGNORE_SUFFIX as i32,
            );
            
            if tag.is_null() {
                break;
            }
            
            let key = std::ffi::CStr::from_ptr((*tag).key)
                .to_string_lossy()
                .to_string();
            let value = std::ffi::CStr::from_ptr((*tag).value)
                .to_string_lossy()
                .to_string();
            
            metadata.insert(key, value);
        }
    }
    
    metadata
}

/// Calculate greatest common divisor for aspect ratio
fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Tauri command to get media information with database caching
#[tauri::command]
pub async fn get_media_info(pool: State<'_, DbPool>, file_path: String) -> Result<MediaInfo, String> {
    get_media_info_with_cache(&pool, &file_path)
        .await
        .map_err(|e| e.to_string())
}

/// Get media information with database caching
async fn get_media_info_with_cache(pool: &DbPool, file_path: &str) -> MediaInfoResult<MediaInfo> {
    debug!("Getting media info for: {}", file_path);
    
    // Get file metadata to check modification time
    let file_metadata = std::fs::metadata(file_path)
        .map_err(|e| MediaInfoError::FileOpen(format!("Cannot access file: {}", e)))?;
    
    let file_modified = file_metadata
        .modified()
        .ok()
        .and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
        })
        .flatten()
        .unwrap_or_else(Utc::now);
    
    // Check if we have cached metadata in the database
    if let Ok(Some(cached_metadata)) = database::get_media_metadata_by_path(pool, file_path).await {
        debug!("Found cached metadata in database");
        
        // Check if file has been modified since last metadata extraction
        if cached_metadata.modified_date >= file_modified {
            debug!("File hasn't changed, using cached metadata");
            
            // Convert cached metadata to MediaInfo format
            if let Some(media_info) = convert_metadata_to_media_info(&cached_metadata) {
                return Ok(media_info);
            } else {
                debug!("Cached metadata incomplete, will re-extract");
            }
        } else {
            debug!("File has been modified since last extraction, will re-extract");
        }
    } else {
        debug!("No cached metadata found, will extract");
    }
    
    // Extract fresh metadata using FFmpeg
    let media_info = extract_media_info(file_path)?;
    
    // Store the extracted metadata in database for future use
    if let Err(e) = store_media_info_in_database(pool, file_path, &media_info, file_modified).await {
        error!("Failed to store metadata in database: {}", e);
        // Continue anyway - we have the metadata even if we can't cache it
    }
    
    Ok(media_info)
}

/// Convert database MediaMetadata to MediaInfo format
fn convert_metadata_to_media_info(metadata: &database::MediaMetadata) -> Option<MediaInfo> {
    // Only return cached data if we have the essential video/audio codec information
    // This ensures we only use cache for complete metadata extractions
    let has_video_info = metadata.video_codec.is_some() && metadata.width.is_some();
    let has_complete_info = has_video_info || metadata.format.is_some();
    
    if !has_complete_info {
        return None;
    }
    
    let video_info = if let (Some(codec), Some(codec_long), Some(width), Some(height)) = (
        &metadata.video_codec,
        &metadata.video_codec_long,
        metadata.width,
        metadata.height,
    ) {
        let aspect_ratio = if height > 0 {
            let gcd = gcd(width as i32, height as i32);
            format!("{}:{}", width / gcd as i64, height / gcd as i64)
        } else {
            "N/A".to_string()
        };
        
        Some(VideoInfo {
            codec: codec.clone(),
            codec_long: codec_long.clone(),
            width: width as i32,
            height: height as i32,
            fps: metadata.frame_rate.unwrap_or(0.0),
            bitrate: metadata.bitrate,
            pix_fmt: "cached".to_string(), // We don't store pixel format in DB yet
            aspect_ratio,
        })
    } else {
        None
    };
    
    let audio_info = if let (Some(codec), Some(codec_long)) = (
        &metadata.audio_codec,
        &metadata.audio_codec_long,
    ) {
        Some(AudioInfo {
            codec: codec.clone(),
            codec_long: codec_long.clone(),
            sample_rate: metadata.sample_rate.unwrap_or(0) as i32,
            channels: metadata.audio_channels.unwrap_or(0) as i32,
            bitrate: metadata.bitrate,
            sample_fmt: "cached".to_string(), // We don't store sample format in DB yet
        })
    } else {
        None
    };
    
    let metadata_map = if let Some(metadata_json) = &metadata.metadata_json {
        serde_json::from_str(metadata_json).unwrap_or_default()
    } else {
        HashMap::new()
    };
    
    Some(MediaInfo {
        video: video_info,
        audio: audio_info,
        general: GeneralInfo {
            format: metadata.format.clone().unwrap_or_else(|| metadata.file_type.clone()),
            format_long: metadata.format.clone().unwrap_or_else(|| {
                format!("{} File", metadata.file_type.to_uppercase())
            }),
            duration: metadata.duration,
            bitrate: metadata.bitrate,
            size: metadata.file_size,
        },
        metadata: metadata_map,
    })
}

/// Store extracted media info in database for caching
async fn store_media_info_in_database(
    pool: &DbPool,
    file_path: &str,
    media_info: &MediaInfo,
    file_modified: DateTime<Utc>,
) -> Result<(), database::DatabaseError> {
    debug!("Storing media info in database for caching");
    
    // Try to find if this file belongs to any scanned folder
    let folder_id = find_folder_id_for_file(pool, file_path).await;
    
    if folder_id.is_none() {
        debug!("No folder_id found for file, using direct SQL insert without FK constraint");
        // Use direct SQL to insert without foreign key constraint
        return insert_metadata_without_folder(pool, file_path, media_info, file_modified).await;
    }
    
    // Serialize additional metadata to JSON
    let metadata_json = if !media_info.metadata.is_empty() {
        Some(serde_json::to_string(&media_info.metadata).unwrap_or_default())
    } else {
        None
    };
    
    // Extract file name from path
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Extract file extension
    let file_type = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    
    let params = InsertMediaParams {
        folder_id: folder_id.unwrap(),
        file_path: file_path.to_string(),
        file_name,
        file_type,
        file_size: media_info.general.size,
        width: media_info.video.as_ref().map(|v| v.width as i64),
        height: media_info.video.as_ref().map(|v| v.height as i64),
        duration: media_info.general.duration,
        created_date: None,
        modified_date: file_modified,
        thumbnail_path: None,
        video_codec: media_info.video.as_ref().map(|v| v.codec.clone()),
        video_codec_long: media_info.video.as_ref().map(|v| v.codec_long.clone()),
        audio_codec: media_info.audio.as_ref().map(|a| a.codec.clone()),
        audio_codec_long: media_info.audio.as_ref().map(|a| a.codec_long.clone()),
        bitrate: media_info.general.bitrate,
        frame_rate: media_info.video.as_ref().map(|v| v.fps),
        sample_rate: media_info.audio.as_ref().map(|a| a.sample_rate as i64),
        audio_channels: media_info.audio.as_ref().map(|a| a.channels as i64),
        format: Some(media_info.general.format.clone()),
        metadata_json,
    };
    
    database::insert_media_metadata(pool, params).await?;
    debug!("Media info stored in database successfully");
    
    Ok(())
}

/// Find the folder_id for a given file path by checking scanned folders
async fn find_folder_id_for_file(pool: &DbPool, file_path: &str) -> Option<i64> {
    // Get all scanned folders and find which one contains this file
    if let Ok(folders) = database::get_all_scanned_folders(pool).await {
        for folder in folders {
            if file_path.starts_with(&folder.path) {
                return Some(folder.id);
            }
        }
    }
    None
}

/// Insert metadata directly with SQL, bypassing the foreign key constraint
async fn insert_metadata_without_folder(
    pool: &DbPool,
    file_path: &str,
    media_info: &MediaInfo,
    file_modified: DateTime<Utc>,
) -> Result<(), database::DatabaseError> {
    let metadata_json = if !media_info.metadata.is_empty() {
        Some(serde_json::to_string(&media_info.metadata).unwrap_or_default())
    } else {
        None
    };
    
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let file_type = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    
    let now = Utc::now();
    
    // First, disable foreign key constraints for this connection
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(pool)
        .await?;
    
    // Insert or update the metadata
    let _ = sqlx::query(
        r#"
        INSERT INTO media_metadata (
            folder_id, file_path, file_name, file_type, file_size,
            width, height, duration, created_date, modified_date,
            thumbnail_path, indexed_at,
            video_codec, video_codec_long, audio_codec, audio_codec_long,
            bitrate, frame_rate, sample_rate, audio_channels, format, metadata_json
        )
        VALUES (0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(file_path) DO UPDATE SET
            modified_date = excluded.modified_date,
            file_size = excluded.file_size,
            width = excluded.width,
            height = excluded.height,
            duration = excluded.duration,
            video_codec = excluded.video_codec,
            video_codec_long = excluded.video_codec_long,
            audio_codec = excluded.audio_codec,
            audio_codec_long = excluded.audio_codec_long,
            bitrate = excluded.bitrate,
            frame_rate = excluded.frame_rate,
            sample_rate = excluded.sample_rate,
            audio_channels = excluded.audio_channels,
            format = excluded.format,
            metadata_json = excluded.metadata_json
        "#,
    )
    .bind(file_path)
    .bind(&file_name)
    .bind(&file_type)
    .bind(media_info.general.size)
    .bind(media_info.video.as_ref().map(|v| v.width as i64))
    .bind(media_info.video.as_ref().map(|v| v.height as i64))
    .bind(media_info.general.duration)
    .bind::<Option<DateTime<Utc>>>(None)
    .bind(file_modified)
    .bind::<Option<&str>>(None)
    .bind(now)
    .bind(media_info.video.as_ref().map(|v| v.codec.as_str()))
    .bind(media_info.video.as_ref().map(|v| v.codec_long.as_str()))
    .bind(media_info.audio.as_ref().map(|a| a.codec.as_str()))
    .bind(media_info.audio.as_ref().map(|a| a.codec_long.as_str()))
    .bind(media_info.general.bitrate)
    .bind(media_info.video.as_ref().map(|v| v.fps))
    .bind(media_info.audio.as_ref().map(|a| a.sample_rate as i64))
    .bind(media_info.audio.as_ref().map(|a| a.channels as i64))
    .bind(Some(media_info.general.format.as_str()))
    .bind(metadata_json.as_deref())
    .execute(pool)
    .await?;
    
    // Re-enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;
    
    debug!("Media info stored in database successfully (without folder)");
    Ok(())
}
