//! FMLM - File & Media Library Manager
//!
//! A Tauri-based application for organizing and browsing photos and videos.
//! Provides recursive directory scanning and media file management capabilities.

pub mod scanner;

// Re-export common types for convenience
pub use scanner::{MediaFile, MediaType, scan_directory};

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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            scanner::scan_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
