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

/// Tauri command to get media information
#[tauri::command]
pub async fn get_media_info(file_path: String) -> Result<MediaInfo, String> {
    extract_media_info(&file_path)
        .map_err(|e| e.to_string())
}
