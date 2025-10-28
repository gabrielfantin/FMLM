//! FMLM - File & Media Library Manager
//!
//! A Tauri-based application for organizing and browsing photos and videos.
//! Provides recursive directory scanning and media file management capabilities.

use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod scanner;
pub mod database;
pub mod db_commands;
pub mod thumbnail;
pub mod thumbnail_commands;
pub mod file_commands;

// Re-export common types for convenience
pub use scanner::{MediaFile, MediaType, scan_directory};
pub use database::{DbPool, initialize_database};

/// Example greeting command for Tauri IPC.
///
/// # Arguments
///
/// * `name` - The name to greet
///
/// # Returns
///
/// Returns a greeting message string.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Initializes and runs the Tauri application.
///
/// Sets up all necessary plugins and command handlers for the application.
/// This is the main entry point for the Tauri runtime.
///
/// # Panics
///
/// Panics if the Tauri application fails to initialize or run. This is expected
/// behavior for application initialization failures.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fmlm=debug,rsmpeg=warn".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize database on app startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                match database::initialize_database(&app_handle).await {
                    Ok(pool) => {
                        println!("Database initialized successfully");
                        app_handle.manage(pool);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize database: {}", e);
                        Err(Box::new(e) as Box<dyn std::error::Error>)
                    }
                }
            })
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            scanner::scan_directory,
            // Database commands
            db_commands::add_scanned_folder,
            db_commands::get_scanned_folders,
            db_commands::delete_scanned_folder,
            db_commands::add_media_metadata,
            db_commands::get_media_by_folder,
            db_commands::get_all_media,
            db_commands::delete_media_metadata,
            db_commands::set_preference,
            db_commands::get_preference,
            db_commands::get_all_preferences,
            db_commands::delete_preference,
            // Thumbnail commands
            thumbnail_commands::generate_thumbnail,
            thumbnail_commands::thumbnail_exists,
            thumbnail_commands::get_thumbnail_path,
            thumbnail_commands::clear_thumbnail_cache,
            thumbnail_commands::get_cache_size,
            thumbnail_commands::generate_thumbnails_batch,
            // File access commands
            file_commands::get_asset_url,
            file_commands::get_mime_type,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
