import { ref, Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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
}

export function useMediaScanner(): MediaScannerState & {
  scanDirectory: (path: string, recursive?: boolean) => Promise<void>
  clearFiles: () => void
} {
  const mediaFiles = ref<MediaFile[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedPath = ref<string | null>(null)

  const scanDirectory = async (path: string, recursive: boolean = true): Promise<void> => {
    try {
      isLoading.value = true
      error.value = null
      selectedPath.value = path

      const result = await invoke<MediaFile[]>('scan_directory', {
        path,
        recursive,
      })

      mediaFiles.value = result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      mediaFiles.value = []
      console.error('Failed to scan directory:', err)
    } finally {
      isLoading.value = false
    }
  }

  const clearFiles = () => {
    mediaFiles.value = []
    error.value = null
    selectedPath.value = null
  }

  return {
    mediaFiles,
    isLoading,
    error,
    selectedPath,
    scanDirectory,
    clearFiles,
  }
}
