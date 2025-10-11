import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'

export interface ThumbnailResponse {
  success: boolean
  thumbnail_path?: string
  thumbnail_data_url?: string
  error?: string
}

/**
 * Composable for managing thumbnails
 */
export function useThumbnails() {
  const thumbnailCache = ref<Map<string, string>>(new Map())
  const loadingThumbnails = ref<Set<string>>(new Set())

  /**
   * Generate a thumbnail for a media file
   * @param filePath - Path to the media file
   * @param isVideo - Whether the file is a video
   * @returns The thumbnail path or null if generation failed
   */
  async function generateThumbnail(
    filePath: string,
    isVideo: boolean
  ): Promise<string | null> {
    try {
      // Check if already cached
      if (thumbnailCache.value.has(filePath)) {
        return thumbnailCache.value.get(filePath)!
      }

      // Mark as loading
      loadingThumbnails.value.add(filePath)

      const response = await invoke<ThumbnailResponse>('generate_thumbnail', {
        filePath,
        isVideo,
      })

      loadingThumbnails.value.delete(filePath)

      if (response.success && response.thumbnail_data_url) {
        // Use data URL directly instead of trying to load via convertFileSrc
        thumbnailCache.value.set(filePath, response.thumbnail_data_url)
        return response.thumbnail_data_url
      }

      return null
    } catch (error) {
      console.error('Failed to generate thumbnail:', error)
      loadingThumbnails.value.delete(filePath)
      return null
    }
  }

  /**
   * Get thumbnail path if it exists in cache
   * @param filePath - Path to the media file
   * @returns The thumbnail URL or null
   */
  async function getThumbnailPath(filePath: string): Promise<string | null> {
    try {
      // Check local cache first
      if (thumbnailCache.value.has(filePath)) {
        return thumbnailCache.value.get(filePath)!
      }

      const thumbnailDataUrl = await invoke<string | null>('get_thumbnail_path', {
        filePath,
      })

      if (thumbnailDataUrl) {
        // Already a data URL, use directly
        thumbnailCache.value.set(filePath, thumbnailDataUrl)
        return thumbnailDataUrl
      }

      return null
    } catch (error) {
      console.error('Failed to get thumbnail path:', error)
      return null
    }
  }

  /**
   * Check if a thumbnail exists for a file
   * @param filePath - Path to the media file
   * @returns True if thumbnail exists
   */
  async function thumbnailExists(filePath: string): Promise<boolean> {
    try {
      return await invoke<boolean>('thumbnail_exists', { filePath })
    } catch (error) {
      console.error('Failed to check thumbnail existence:', error)
      return false
    }
  }

  /**
   * Generate thumbnails for multiple files in batch
   * @param files - Array of [filePath, isVideo] tuples
   * @returns Array of thumbnail responses
   */
  async function generateThumbnailsBatch(
    files: [string, boolean][]
  ): Promise<ThumbnailResponse[]> {
    try {
      // Mark all as loading
      files.forEach(([filePath]) => loadingThumbnails.value.add(filePath))

      const responses = await invoke<ThumbnailResponse[]>(
        'generate_thumbnails_batch',
        { files }
      )

      // Update cache and remove from loading
      responses.forEach((response, index) => {
        const [filePath] = files[index]
        loadingThumbnails.value.delete(filePath)

        if (response.success && response.thumbnail_data_url) {
          // Use data URL directly
          thumbnailCache.value.set(filePath, response.thumbnail_data_url)
        }
      })

      return responses
    } catch (error) {
      console.error('Failed to generate thumbnails batch:', error)
      // Clear loading states
      files.forEach(([filePath]) => loadingThumbnails.value.delete(filePath))
      return []
    }
  }

  /**
   * Clear all cached thumbnails
   */
  async function clearCache(): Promise<void> {
    try {
      await invoke('clear_thumbnail_cache')
      thumbnailCache.value.clear()
    } catch (error) {
      console.error('Failed to clear thumbnail cache:', error)
      throw error
    }
  }

  /**
   * Get the size of the thumbnail cache in bytes
   */
  async function getCacheSize(): Promise<number> {
    try {
      return await invoke<number>('get_cache_size')
    } catch (error) {
      console.error('Failed to get cache size:', error)
      return 0
    }
  }

  /**
   * Check if a thumbnail is currently loading
   */
  const isThumbnailLoading = computed(() => {
    return (filePath: string) => loadingThumbnails.value.has(filePath)
  })

  return {
    generateThumbnail,
    getThumbnailPath,
    thumbnailExists,
    generateThumbnailsBatch,
    clearCache,
    getCacheSize,
    isThumbnailLoading,
    thumbnailCache,
  }
}
