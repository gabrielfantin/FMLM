// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Main entry point for the FMLM application.
///
/// Initializes and runs the Tauri application by delegating to the library's run function.
fn main() {
    fmlm_lib::run()
}
