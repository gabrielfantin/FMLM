/**
 * Composable for database operations
 * 
 * Provides TypeScript-friendly wrappers around Tauri database commands
 * for managing scanned folders, media metadata, and user preferences.
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

export interface ScannedFolder {
  id: number;
  path: string;
  name: string;
  last_scanned: string;
  file_count: number;
  created_at: string;
}

export interface MediaMetadata {
  id: number;
  folder_id: number;
  file_path: string;
  file_name: string;
  file_type: string;
  file_size: number;
  width: number | null;
  height: number | null;
  duration: number | null;
  created_date: string | null;
  modified_date: string;
  thumbnail_path: string | null;
  indexed_at: string;
}

export interface InsertMediaParams {
  folder_id: number;
  file_path: string;
  file_name: string;
  file_type: string;
  file_size: number;
  width?: number;
  height?: number;
  duration?: number;
  created_date?: string;
  modified_date: string;
  thumbnail_path?: string;
}

export interface UserPreference {
  key: string;
  value: string;
  updated_at: string;
}

// ============================================================================
// Database Operations
// ============================================================================

export function useDatabase() {
  // ------------------------------------------------------------------------
  // Scanned Folders
  // ------------------------------------------------------------------------

  /**
   * Add or update a scanned folder
   */
  const addScannedFolder = async (
    path: string,
    name: string,
    fileCount: number
  ): Promise<number> => {
    return invoke<number>('add_scanned_folder', {
      path,
      name,
      fileCount,
    });
  };

  /**
   * Get all scanned folders
   */
  const getScannedFolders = async (): Promise<ScannedFolder[]> => {
    return invoke<ScannedFolder[]>('get_scanned_folders');
  };

  /**
   * Delete a scanned folder and its associated media
   */
  const deleteScannedFolder = async (folderId: number): Promise<void> => {
    return invoke<void>('delete_scanned_folder', { folderId });
  };

  // ------------------------------------------------------------------------
  // Media Metadata
  // ------------------------------------------------------------------------

  /**
   * Add media file metadata to the database
   */
  const addMediaMetadata = async (params: InsertMediaParams): Promise<number> => {
    return invoke<number>('add_media_metadata', { params });
  };

  /**
   * Get media metadata for a specific folder
   */
  const getMediaByFolder = async (folderId: number): Promise<MediaMetadata[]> => {
    return invoke<MediaMetadata[]>('get_media_by_folder', { folderId });
  };

  /**
   * Get all media metadata
   */
  const getAllMedia = async (): Promise<MediaMetadata[]> => {
    return invoke<MediaMetadata[]>('get_all_media');
  };

  /**
   * Delete media metadata by file path
   */
  const deleteMediaMetadata = async (filePath: string): Promise<void> => {
    return invoke<void>('delete_media_metadata', { filePath });
  };

  // ------------------------------------------------------------------------
  // User Preferences
  // ------------------------------------------------------------------------

  /**
   * Set a user preference
   */
  const setPreference = async (key: string, value: string): Promise<void> => {
    return invoke<void>('set_preference', { key, value });
  };

  /**
   * Get a user preference by key
   */
  const getPreference = async (key: string): Promise<string | null> => {
    return invoke<string | null>('get_preference', { key });
  };

  /**
   * Get all user preferences
   */
  const getAllPreferences = async (): Promise<UserPreference[]> => {
    return invoke<UserPreference[]>('get_all_preferences');
  };

  /**
   * Delete a user preference
   */
  const deletePreference = async (key: string): Promise<void> => {
    return invoke<void>('delete_preference', { key });
  };

  return {
    // Folders
    addScannedFolder,
    getScannedFolders,
    deleteScannedFolder,
    // Media
    addMediaMetadata,
    getMediaByFolder,
    getAllMedia,
    deleteMediaMetadata,
    // Preferences
    setPreference,
    getPreference,
    getAllPreferences,
    deletePreference,
  };
}
