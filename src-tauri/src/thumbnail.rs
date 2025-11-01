//! Thumbnail generation and caching module
//!
//! Handles thumbnail generation for images and videos with persistent caching.
//! Uses SHA-256 hashes of file paths as cache keys to avoid regenerating thumbnails.

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::fs;
use std::ffi::CString;
use thiserror::Error;
use tracing::{info, warn, error, debug, instrument};

/// Thumbnail dimensions (width x height)
const THUMBNAIL_SIZE: u32 = 256;

/// Error types for thumbnail operations
#[derive(Debug, Error)]
pub enum ThumbnailError {
    #[error("Failed to open image: {0}")]
    ImageOpen(#[from] image::ImageError),
    
    #[error("Failed to decode video frame: {0}")]
    VideoDecoding(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unsupported file format")]
    UnsupportedFormat,
    
    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),
}

pub type ThumbnailResult<T> = Result<T, ThumbnailError>;

/// Get the thumbnail cache directory for the application
pub fn get_cache_dir() -> ThumbnailResult<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| ThumbnailError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine cache directory"
        )))?
        .join("fmlm")
        .join("thumbnails");
    
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    
    Ok(cache_dir)
}

/// Generate a cache key from a file path using SHA-256
fn generate_cache_key(file_path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_path.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get the cached thumbnail path for a file
pub fn get_thumbnail_path(file_path: &str) -> ThumbnailResult<PathBuf> {
    let cache_dir = get_cache_dir()?;
    let cache_key = generate_cache_key(file_path);
    Ok(cache_dir.join(format!("{}.jpg", cache_key)))
}

/// Check if a thumbnail exists in cache
pub fn thumbnail_exists(file_path: &str) -> ThumbnailResult<bool> {
    let thumbnail_path = get_thumbnail_path(file_path)?;
    Ok(thumbnail_path.exists())
}

/// Resize an image to thumbnail size maintaining aspect ratio
fn resize_to_thumbnail(img: DynamicImage) -> DynamicImage {
    img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3)
}

/// Generate a thumbnail for an image file
#[instrument(skip_all, fields(file_path = %file_path))]
pub fn generate_image_thumbnail(file_path: &str) -> ThumbnailResult<String> {
    debug!("Starting image thumbnail generation");
    let thumbnail_path = get_thumbnail_path(file_path)?;
    
    if thumbnail_path.exists() {
        debug!("Thumbnail already exists in cache");
        return Ok(thumbnail_path.to_string_lossy().to_string());
    }
    
    info!("Generating new image thumbnail");
    let img = image::open(file_path)
        .map_err(|e| {
            error!("Failed to open image: {}", e);
            e
        })?;
    
    let thumbnail = resize_to_thumbnail(img);
    thumbnail.save_with_format(&thumbnail_path, ImageFormat::Jpeg)
        .map_err(|e| {
            error!("Failed to save thumbnail: {}", e);
            e
        })?;
    
    info!("Image thumbnail generated successfully");
    Ok(thumbnail_path.to_string_lossy().to_string())
}

/// Generate a thumbnail for a video file by extracting a frame
#[instrument(skip_all, fields(file_path = %file_path))]
pub fn generate_video_thumbnail(file_path: &str) -> ThumbnailResult<String> {
    use rsmpeg::avcodec::{AVCodec, AVCodecContext};
    use rsmpeg::avformat::AVFormatContextInput;
    use rsmpeg::avutil::AVFrame;
    use rsmpeg::ffi;
    use rsmpeg::swscale::SwsContext;
    
    debug!("Starting video thumbnail generation");
    let thumbnail_path = get_thumbnail_path(file_path)?;
    
    if thumbnail_path.exists() {
        debug!("Video thumbnail already exists in cache");
        return Ok(thumbnail_path.to_string_lossy().to_string());
    }
    
    info!("Generating new video thumbnail");
    let c_file_path = CString::new(file_path)
        .map_err(|_e| {
            error!("Invalid file path (contains null bytes)");
            ThumbnailError::VideoDecoding("Invalid file path".to_string())
        })?;
    
    debug!("Opening video file");
    let mut input_context = AVFormatContextInput::open(&c_file_path)
        .map_err(|e| {
            error!("Failed to open video file: {:?}", e);
            ThumbnailError::VideoDecoding(format!("Failed to open video: {:?}", e))
        })?;
    
    debug!("Reading stream info");
    input_context.dump(0, &c_file_path)
        .map_err(|e| {
            error!("Failed to read stream info: {:?}", e);
            ThumbnailError::VideoDecoding(format!("Failed to read stream info: {:?}", e))
        })?;
    
    debug!("Finding video stream and setting up decoder");
    let (stream_index, mut decoder) = {
        let (stream_index, stream) = input_context
            .streams()
            .into_iter()
            .enumerate()
            .find(|(_, stream)| {
                stream.codecpar().codec_type == ffi::AVMEDIA_TYPE_VIDEO
            })
            .ok_or_else(|| {
                error!("No video stream found in file");
                ThumbnailError::VideoDecoding("No video stream found".to_string())
            })?;
        
        debug!("Found video stream at index {}", stream_index);
        let codec_id = stream.codecpar().codec_id;
        let decoder = AVCodec::find_decoder(codec_id)
            .ok_or_else(|| {
                error!("Failed to find decoder for codec");
                ThumbnailError::VideoDecoding("Failed to find decoder".to_string())
            })?;
        
        let mut decoder_context = AVCodecContext::new(&decoder);
        decoder_context
            .apply_codecpar(&stream.codecpar())
            .map_err(|e| {
                error!("Failed to copy codec parameters: {:?}", e);
                ThumbnailError::VideoDecoding(format!("Failed to copy codec parameters: {:?}", e))
            })?;
        
        decoder_context
            .open(None)
            .map_err(|e| {
                error!("Failed to open decoder: {:?}", e);
                ThumbnailError::VideoDecoding(format!("Failed to open decoder: {:?}", e))
            })?;
        
        debug!("Decoder opened successfully");
        (stream_index, decoder_context)
    };
    
    let duration = input_context.duration;
    if duration > 0 {
        let seek_target = (duration as f64 * 0.1) as i64;
        let _ = unsafe {
            ffi::avformat_seek_file(
                input_context.as_mut_ptr(),
                stream_index as i32,
                i64::MIN,
                seek_target,
                seek_target,
                0,
            )
        };
    }
    
    let src_width = decoder.width as i32;
    let src_height = decoder.height as i32;
    let src_format = decoder.pix_fmt;
    
    let mut scaler = SwsContext::get_context(
        src_width,
        src_height,
        src_format,
        src_width,
        src_height,
        ffi::AV_PIX_FMT_RGB24,
        ffi::SWS_BILINEAR,
        None,
        None,
        None,
    ).ok_or_else(|| ThumbnailError::VideoDecoding("Failed to create scaler".to_string()))?;
    
    let mut frame_found = false;
    let mut rgb_data: Option<Vec<u8>> = None;
    let mut packets_checked = 0;
    const MAX_PACKETS: usize = 100;
    
    while let Ok(packet) = input_context.read_packet() {
        if let Some(pkt) = packet {
            if packets_checked >= MAX_PACKETS {
                break;
            }
            packets_checked += 1;
            
            if pkt.stream_index != stream_index as i32 {
                continue;
            }
            
            decoder
                .send_packet(Some(&pkt))
                .map_err(|e| ThumbnailError::VideoDecoding(format!("Failed to send packet: {:?}", e)))?;
            
            loop {
                match decoder.receive_frame() {
                    Ok(decoded_frame) => {
                        let mut rgb_frame = AVFrame::new();
                        rgb_frame.set_width(src_width);
                        rgb_frame.set_height(src_height);
                        rgb_frame.set_format(ffi::AV_PIX_FMT_RGB24);
                        
                        rgb_frame
                            .alloc_buffer()
                            .map_err(|e| ThumbnailError::VideoDecoding(format!("Failed to allocate frame: {:?}", e)))?;
                        
                        scaler
                            .scale_frame(&decoded_frame, 0, src_height, &mut rgb_frame)
                            .map_err(|e| ThumbnailError::VideoDecoding(format!("Failed to scale frame: {:?}", e)))?;
                        
                        let buffer_size = unsafe {
                            ffi::av_image_get_buffer_size(
                                ffi::AV_PIX_FMT_RGB24,
                                src_width,
                                src_height,
                                1,
                            )
                        };
                        
                        if buffer_size > 0 {
                            let mut buffer = vec![0u8; buffer_size as usize];
                            unsafe {
                                ffi::av_image_copy_to_buffer(
                                    buffer.as_mut_ptr(),
                                    buffer_size,
                                    rgb_frame.data.as_ptr() as *const *const u8,
                                    rgb_frame.linesize.as_ptr(),
                                    ffi::AV_PIX_FMT_RGB24,
                                    src_width,
                                    src_height,
                                    1,
                                );
                            }
                            rgb_data = Some(buffer);
                            frame_found = true;
                        }
                        break;
                    }
                    Err(rsmpeg::error::RsmpegError::DecoderDrainError)
                    | Err(rsmpeg::error::RsmpegError::DecoderFlushedError) => break,
                    Err(_) => break,
                }
            }
            
            if frame_found {
                break;
            }
        } else {
            break;
        }
    }
    
    debug!("Flushing decoder");
    let _ = decoder.send_packet(None);
    
    if let Some(buffer) = rgb_data {
        info!("Successfully decoded video frame, creating thumbnail");
        let img = image::RgbImage::from_raw(src_width as u32, src_height as u32, buffer)
            .ok_or_else(|| {
                error!("Failed to create image from decoded frame data");
                ThumbnailError::VideoDecoding("Failed to create image from frame".to_string())
            })?;
        
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let thumbnail = resize_to_thumbnail(dynamic_img);
        thumbnail.save_with_format(&thumbnail_path, ImageFormat::Jpeg)
            .map_err(|e| {
                error!("Failed to save video thumbnail: {}", e);
                e
            })?;
        
        info!("Video thumbnail generated successfully");
        Ok(thumbnail_path.to_string_lossy().to_string())
    } else {
        error!("No frames could be decoded from video after checking {} packets", packets_checked);
        Err(ThumbnailError::VideoDecoding("No frames could be decoded".to_string()))
    }
}

/// Generate a thumbnail for any supported media file
pub fn generate_thumbnail(file_path: &str, is_video: bool) -> ThumbnailResult<String> {
    if is_video {
        generate_video_thumbnail(file_path)
    } else {
        generate_image_thumbnail(file_path)
    }
}

/// Clear all cached thumbnails
pub fn clear_cache() -> ThumbnailResult<()> {
    let cache_dir = get_cache_dir()?;
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
        fs::create_dir_all(&cache_dir)?;
    }
    Ok(())
}

/// Get the size of the thumbnail cache in bytes
pub fn get_cache_size() -> ThumbnailResult<u64> {
    let cache_dir = get_cache_dir()?;
    let mut total_size = 0u64;
    
    if cache_dir.exists() {
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }
    }
    
    Ok(total_size)
}
