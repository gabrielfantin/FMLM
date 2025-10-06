//! Database module for FMLM application
//!
//! Handles SQLite database operations for storing media metadata, user preferences,
//! and folder information. Uses sqlx for async database operations with connection pooling.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{FromRow, Row};
use std::str::FromStr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::Manager;

/// Application database connection pool
pub type DbPool = SqlitePool;

/// Error type for database operations
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Represents a scanned folder in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScannedFolder {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub last_scanned: DateTime<Utc>,
    pub file_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Represents media file metadata stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaMetadata {
    pub id: i64,
    pub folder_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration: Option<f64>,
    pub created_date: Option<DateTime<Utc>>,
    pub modified_date: DateTime<Utc>,
    pub thumbnail_path: Option<String>,
    pub indexed_at: DateTime<Utc>,
}

/// Parameters for inserting media metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertMediaParams {
    pub folder_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration: Option<f64>,
    pub created_date: Option<DateTime<Utc>>,
    pub modified_date: DateTime<Utc>,
    pub thumbnail_path: Option<String>,
}

/// Represents user preferences
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

/// Initializes the database connection pool and runs migrations
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle to get the data directory
///
/// # Returns
///
/// Returns a connection pool ready for use
///
/// # Errors
///
/// Returns `DatabaseError` if the database cannot be initialized or migrations fail
pub async fn initialize_database(app_handle: &tauri::AppHandle) -> DatabaseResult<DbPool> {
    // Get the app data directory from Tauri
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| DatabaseError::Migration(format!("Failed to get app data directory: {}", e)))?;
    
    // Create the data directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)?;
    
    // Construct the database path
    let db_path = app_data_dir.join("fmlm.db");
    let db_url = format!("sqlite:{}", db_path.display());
    
    println!("Initializing database at: {}", db_path.display());
    
    // Configure SQLite connection options
    let connect_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    
    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    Ok(pool)
}

/// Runs database migrations to create or update schema
async fn run_migrations(pool: &DbPool) -> DatabaseResult<()> {
    // Create scanned_folders table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scanned_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            last_scanned DATETIME NOT NULL,
            file_count INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    // Create media_metadata table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS media_metadata (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            width INTEGER,
            height INTEGER,
            duration REAL,
            created_date DATETIME,
            modified_date DATETIME NOT NULL,
            thumbnail_path TEXT,
            indexed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (folder_id) REFERENCES scanned_folders(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    // Create indexes for better query performance
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_media_folder_id 
        ON media_metadata(folder_id)
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_media_file_type 
        ON media_metadata(file_type)
        "#,
    )
    .execute(pool)
    .await?;
    
    // Create user_preferences table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_preferences (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    println!("Database migrations completed successfully");
    
    Ok(())
}

// ============================================================================
// Scanned Folders Operations
// ============================================================================

/// Adds or updates a scanned folder in the database
pub async fn upsert_scanned_folder(
    pool: &DbPool,
    path: &str,
    name: &str,
    file_count: i64,
) -> DatabaseResult<i64> {
    let now = Utc::now();
    
    let result = sqlx::query(
        r#"
        INSERT INTO scanned_folders (path, name, last_scanned, file_count, created_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
            last_scanned = excluded.last_scanned,
            file_count = excluded.file_count
        RETURNING id
        "#,
    )
    .bind(path)
    .bind(name)
    .bind(now)
    .bind(file_count)
    .bind(now)
    .fetch_one(pool)
    .await?;
    
    Ok(result.get(0))
}

/// Retrieves all scanned folders
pub async fn get_all_scanned_folders(pool: &DbPool) -> DatabaseResult<Vec<ScannedFolder>> {
    let folders = sqlx::query_as::<_, ScannedFolder>(
        r#"
        SELECT id, path, name, last_scanned, file_count, created_at
        FROM scanned_folders
        ORDER BY last_scanned DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    
    Ok(folders)
}

/// Deletes a scanned folder and its associated media metadata
pub async fn delete_scanned_folder(pool: &DbPool, folder_id: i64) -> DatabaseResult<()> {
    sqlx::query("DELETE FROM scanned_folders WHERE id = ?")
        .bind(folder_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

// ============================================================================
// Media Metadata Operations
// ============================================================================

/// Adds media file metadata to the database
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `params` - Media metadata parameters
pub async fn insert_media_metadata(
    pool: &DbPool,
    params: InsertMediaParams,
) -> DatabaseResult<i64> {
    let now = Utc::now();
    
    let result = sqlx::query(
        r#"
        INSERT INTO media_metadata (
            folder_id, file_path, file_name, file_type, file_size,
            width, height, duration, created_date, modified_date,
            thumbnail_path, indexed_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(file_path) DO UPDATE SET
            modified_date = excluded.modified_date,
            file_size = excluded.file_size,
            width = excluded.width,
            height = excluded.height,
            duration = excluded.duration,
            thumbnail_path = excluded.thumbnail_path
        RETURNING id
        "#,
    )
    .bind(params.folder_id)
    .bind(&params.file_path)
    .bind(&params.file_name)
    .bind(&params.file_type)
    .bind(params.file_size)
    .bind(params.width)
    .bind(params.height)
    .bind(params.duration)
    .bind(params.created_date)
    .bind(params.modified_date)
    .bind(params.thumbnail_path.as_deref())
    .bind(now)
    .fetch_one(pool)
    .await?;
    
    Ok(result.get(0))
}

/// Retrieves media metadata for a specific folder
pub async fn get_media_by_folder(pool: &DbPool, folder_id: i64) -> DatabaseResult<Vec<MediaMetadata>> {
    let media = sqlx::query_as::<_, MediaMetadata>(
        r#"
        SELECT id, folder_id, file_path, file_name, file_type, file_size,
               width, height, duration, created_date, modified_date,
               thumbnail_path, indexed_at
        FROM media_metadata
        WHERE folder_id = ?
        ORDER BY file_name ASC
        "#,
    )
    .bind(folder_id)
    .fetch_all(pool)
    .await?;
    
    Ok(media)
}

/// Retrieves all media metadata
pub async fn get_all_media(pool: &DbPool) -> DatabaseResult<Vec<MediaMetadata>> {
    let media = sqlx::query_as::<_, MediaMetadata>(
        r#"
        SELECT id, folder_id, file_path, file_name, file_type, file_size,
               width, height, duration, created_date, modified_date,
               thumbnail_path, indexed_at
        FROM media_metadata
        ORDER BY indexed_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    
    Ok(media)
}

/// Deletes media metadata for files that no longer exist
pub async fn delete_media_metadata(pool: &DbPool, file_path: &str) -> DatabaseResult<()> {
    sqlx::query("DELETE FROM media_metadata WHERE file_path = ?")
        .bind(file_path)
        .execute(pool)
        .await?;
    
    Ok(())
}

// ============================================================================
// User Preferences Operations
// ============================================================================

/// Sets a user preference
pub async fn set_preference(pool: &DbPool, key: &str, value: &str) -> DatabaseResult<()> {
    let now = Utc::now();
    
    sqlx::query(
        r#"
        INSERT INTO user_preferences (key, value, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(key)
    .bind(value)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Gets a user preference by key
pub async fn get_preference(pool: &DbPool, key: &str) -> DatabaseResult<Option<String>> {
    let result = sqlx::query_as::<_, UserPreference>(
        "SELECT key, value, updated_at FROM user_preferences WHERE key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;
    
    Ok(result.map(|pref| pref.value))
}

/// Gets all user preferences
pub async fn get_all_preferences(pool: &DbPool) -> DatabaseResult<Vec<UserPreference>> {
    let prefs = sqlx::query_as::<_, UserPreference>(
        "SELECT key, value, updated_at FROM user_preferences ORDER BY key ASC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(prefs)
}

/// Deletes a user preference
pub async fn delete_preference(pool: &DbPool, key: &str) -> DatabaseResult<()> {
    sqlx::query("DELETE FROM user_preferences WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await?;
    
    Ok(())
}
