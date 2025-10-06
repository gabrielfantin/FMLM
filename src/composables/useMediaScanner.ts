import { ref, Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDatabase } from './useDatabase'

export type MediaType = 'image' | 'video' | 'unknown'

export interface MediaFile {
  path: string
  name: string
  size: number
  modified: number
  file_type: string
  media_type: MediaType
}

export interface MediaScannerState {
  mediaFiles: Ref<MediaFile[]>
  isLoading: Ref<boolean>
  error: Ref<string | null>
  selectedPath: Ref<string | null>
  selectedFolderId: Ref<number | null>
}

export function useMediaScanner(): MediaScannerState & {
  scanDirectory: (path: string, recursive?: boolean) => Promise<number | null>
  clearFiles: () => void
} {
  const mediaFiles = ref<MediaFile[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedPath = ref<string | null>(null)
  const selectedFolderId = ref<number | null>(null)

  const db = useDatabase()

  /**
   * Scan a directory and save it to the database
   * Returns the folder ID from the database
   */
  const scanDirectory = async (path: string, recursive: boolean = true): Promise<number | null> => {
    try {
      isLoading.value = true
      error.value = null
      selectedPath.value = path

      // Scan the directory for media files
      const result = await invoke<MediaFile[]>('scan_directory', {
        path,
        recursive,
      })

      mediaFiles.value = result

      // Extract folder name from path
      const folderName = path.split(/[\\/]/).filter(Boolean).pop() || path

      // Save or update the folder in the database
      const folderId = await db.addScannedFolder(
        path,
        folderName,
        result.length
      )

      selectedFolderId.value = folderId

      return folderId
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      mediaFiles.value = []
      selectedFolderId.value = null
      console.error('Failed to scan directory:', err)
      return null
    } finally {
      isLoading.value = false
    }
  }

  const clearFiles = () => {
    mediaFiles.value = []
    error.value = null
    selectedPath.value = null
    selectedFolderId.value = null
  }

  return {
    mediaFiles,
    isLoading,
    error,
    selectedPath,
    selectedFolderId,
    scanDirectory,
    clearFiles,
  }
}
