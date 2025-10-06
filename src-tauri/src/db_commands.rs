//! Tauri commands for database operations
//!
//! Provides IPC commands for frontend to interact with the SQLite database.

use crate::database::{
    self, DbPool, InsertMediaParams, MediaMetadata, ScannedFolder, UserPreference,
};
use tauri::State;

/// Error type for command operations
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    message: String,
}

impl From<database::DatabaseError> for CommandError {
    fn from(err: database::DatabaseError) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

/// Type alias for command results
type CommandResult<T> = Result<T, CommandError>;

// ============================================================================
// Scanned Folders Commands
// ============================================================================

/// Adds or updates a scanned folder
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `path` - Full path to the folder
/// * `name` - Display name of the folder
/// * `file_count` - Number of files in the folder
///
/// # Returns
///
/// Returns the folder ID
#[tauri::command]
pub async fn add_scanned_folder(
    pool: State<'_, DbPool>,
    path: String,
    name: String,
    file_count: i64,
) -> CommandResult<i64> {
    let folder_id = database::upsert_scanned_folder(&pool, &path, &name, file_count).await?;
    Ok(folder_id)
}

/// Retrieves all scanned folders
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
///
/// # Returns
///
/// Returns a list of all scanned folders
#[tauri::command]
pub async fn get_scanned_folders(pool: State<'_, DbPool>) -> CommandResult<Vec<ScannedFolder>> {
    let folders = database::get_all_scanned_folders(&pool).await?;
    Ok(folders)
}

/// Deletes a scanned folder and its associated media
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `folder_id` - ID of the folder to delete
#[tauri::command]
pub async fn delete_scanned_folder(
    pool: State<'_, DbPool>,
    folder_id: i64,
) -> CommandResult<()> {
    database::delete_scanned_folder(&pool, folder_id).await?;
    Ok(())
}

// ============================================================================
// Media Metadata Commands
// ============================================================================

/// Adds media file metadata to the database
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `params` - Media metadata parameters (folder_id, file info, dimensions, etc.)
///
/// # Returns
///
/// Returns the media metadata ID
#[tauri::command]
pub async fn add_media_metadata(
    pool: State<'_, DbPool>,
    params: InsertMediaParams,
) -> CommandResult<i64> {
    let media_id = database::insert_media_metadata(&pool, params).await?;
    Ok(media_id)
}

/// Retrieves media metadata for a specific folder
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `folder_id` - ID of the folder
///
/// # Returns
///
/// Returns a list of media metadata for the folder
#[tauri::command]
pub async fn get_media_by_folder(
    pool: State<'_, DbPool>,
    folder_id: i64,
) -> CommandResult<Vec<MediaMetadata>> {
    let media = database::get_media_by_folder(&pool, folder_id).await?;
    Ok(media)
}

/// Retrieves all media metadata
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
///
/// # Returns
///
/// Returns a list of all media metadata
#[tauri::command]
pub async fn get_all_media(pool: State<'_, DbPool>) -> CommandResult<Vec<MediaMetadata>> {
    let media = database::get_all_media(&pool).await?;
    Ok(media)
}

/// Deletes media metadata by file path
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `file_path` - Full path to the media file
#[tauri::command]
pub async fn delete_media_metadata(
    pool: State<'_, DbPool>,
    file_path: String,
) -> CommandResult<()> {
    database::delete_media_metadata(&pool, &file_path).await?;
    Ok(())
}

// ============================================================================
// User Preferences Commands
// ============================================================================

/// Sets a user preference
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `key` - Preference key
/// * `value` - Preference value
#[tauri::command]
pub async fn set_preference(
    pool: State<'_, DbPool>,
    key: String,
    value: String,
) -> CommandResult<()> {
    database::set_preference(&pool, &key, &value).await?;
    Ok(())
}

/// Gets a user preference by key
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `key` - Preference key
///
/// # Returns
///
/// Returns the preference value if found, None otherwise
#[tauri::command]
pub async fn get_preference(
    pool: State<'_, DbPool>,
    key: String,
) -> CommandResult<Option<String>> {
    let value = database::get_preference(&pool, &key).await?;
    Ok(value)
}

/// Gets all user preferences
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
///
/// # Returns
///
/// Returns a list of all user preferences
#[tauri::command]
pub async fn get_all_preferences(pool: State<'_, DbPool>) -> CommandResult<Vec<UserPreference>> {
    let prefs = database::get_all_preferences(&pool).await?;
    Ok(prefs)
}

/// Deletes a user preference
///
/// # Arguments
///
/// * `pool` - Database connection pool (injected by Tauri)
/// * `key` - Preference key to delete
#[tauri::command]
pub async fn delete_preference(pool: State<'_, DbPool>, key: String) -> CommandResult<()> {
    database::delete_preference(&pool, &key).await?;
    Ok(())
}
