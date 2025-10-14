<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Image, Play, FileType, Calendar, HardDrive, Info, Loader2 } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { MediaFile } from '../composables/useMediaScanner'
import { useResizable } from '@/composables/useResizable'

interface Props {
  selectedFile?: MediaFile | null
  isCollapsed?: boolean
}

const props = defineProps<Props>()

// Resizable panel
const { width: panelWidth, isResizing, startResize } = useResizable({
  defaultWidth: 384, // 96 * 4 = 384px (w-96)
  minWidth: 300,
  maxWidthPercent: 70, // 70% of screen width
  side: 'right'
})

// Compute panel width style
const panelStyle = computed(() => {
  if (props.isCollapsed) return { width: '0px' }
  return { width: `${panelWidth.value}px` }
})

const mediaUrl = ref<string | null>(null)
const isLoadingMedia = ref(false)
const loadError = ref<string | null>(null)

// Format file size for display
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}

// Format timestamp to readable date
function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  return date.toLocaleString()
}

// Get short file name (last part of path)
const fileName = computed(() => props.selectedFile?.name || 'No file selected')

// Get file extension
const fileExtension = computed(() => {
  if (!props.selectedFile) return ''
  return props.selectedFile.file_type.toUpperCase()
})

// Load the actual media file using Tauri's asset protocol (supports streaming)
async function loadMediaFile() {
  if (!props.selectedFile) {
    mediaUrl.value = null
    return
  }

  isLoadingMedia.value = true
  loadError.value = null

  try {
    // Verify the file exists through the backend
    await invoke<string>('get_asset_url', {
      filePath: props.selectedFile.path
    })

    // Convert the file path to an asset URL that supports streaming
    // This is crucial for video files as it enables range requests
    mediaUrl.value = convertFileSrc(props.selectedFile.path)
  } catch (error) {
    console.error('Failed to load media file:', error)
    loadError.value = error instanceof Error ? error.message : String(error)
    mediaUrl.value = null
  } finally {
    isLoadingMedia.value = false
  }
}

// Watch for changes in selected file
watch(() => props.selectedFile, () => {
  loadMediaFile()
}, { immediate: true })

// Expose width so parent can use it
defineExpose({
  width: panelWidth,
})
</script>

<template>
  <aside 
    :class="[
      'relative bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 flex flex-col h-screen',
      isCollapsed ? 'overflow-hidden' : ''
    ]"
    :style="panelStyle"
  >
    <!-- Resize Handle -->
    <div
      v-if="!isCollapsed"
      @mousedown="startResize"
      class="absolute top-0 left-0 w-1 h-full cursor-col-resize hover:bg-indigo-500 transition-colors z-50 group"
      :class="{ 'bg-indigo-500': isResizing }"
    >
      <div class="absolute inset-y-0 -left-1 w-3"></div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- No Selection State -->
      <div v-if="!selectedFile" class="flex flex-col items-center justify-center py-12 px-4 text-center h-full">
        <Info :size="48" class="text-gray-400 mb-4" />
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
          No file selected
        </p>
        <p class="text-xs text-gray-500 dark:text-gray-500">
          Select a file to view its details
        </p>
      </div>

      <!-- File Selected -->
      <div v-else class="p-3 space-y-3">
        <!-- Media Preview -->
        <div>
          <div class="relative bg-gray-100 dark:bg-gray-900 rounded-lg overflow-hidden aspect-video">
            <!-- Loading State -->
            <div v-if="isLoadingMedia" class="absolute inset-0 flex items-center justify-center">
              <Loader2 :size="32" class="animate-spin text-indigo-600" />
            </div>

            <!-- Error State -->
            <div v-else-if="loadError" class="absolute inset-0 flex flex-col items-center justify-center p-4 text-center">
              <Info :size="32" class="text-red-500 mb-2" />
              <p class="text-xs text-red-600 dark:text-red-400">Failed to load media</p>
              <p class="text-xs text-gray-500 mt-1">{{ loadError }}</p>
            </div>

            <!-- Image Preview -->
            <div v-else-if="mediaUrl && selectedFile.media_type === 'image'" class="w-full h-full">
              <img 
                :src="mediaUrl" 
                :alt="fileName"
                class="w-full h-full object-contain"
              />
            </div>

            <!-- Video Preview -->
            <div v-else-if="mediaUrl && selectedFile.media_type === 'video'" class="w-full h-full">
              <video 
                :src="mediaUrl" 
                controls
                class="w-full h-full object-contain"
              >
                Your browser does not support the video tag.
              </video>
            </div>

            <!-- Placeholder -->
            <div v-else class="absolute inset-0 flex items-center justify-center">
              <Image v-if="selectedFile.media_type === 'image'" :size="48" class="text-gray-400" />
              <Play v-else :size="48" class="text-gray-400" />
            </div>
          </div>
        </div>

        <!-- File Information -->
        <div class="space-y-3">
          <!-- File Name -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <FileType :size="16" />
              <span class="text-xs font-medium uppercase">Name</span>
            </div>
            <p class="text-sm text-gray-900 dark:text-white break-all">
              {{ fileName }}
            </p>
          </div>

          <!-- File Type -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <FileType :size="16" />
              <span class="text-xs font-medium uppercase">Type</span>
            </div>
            <div class="flex items-center gap-2">
              <span 
                class="inline-block px-2 py-1 text-xs font-semibold text-white rounded"
                :class="{
                  'bg-blue-500': selectedFile.media_type === 'image',
                  'bg-red-500': selectedFile.media_type === 'video',
                  'bg-gray-500': selectedFile.media_type === 'unknown'
                }"
              >
                {{ fileExtension }}
              </span>
              <span class="text-sm text-gray-600 dark:text-gray-400">
                {{ selectedFile.media_type }}
              </span>
            </div>
          </div>

          <!-- File Size -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <HardDrive :size="16" />
              <span class="text-xs font-medium uppercase">Size</span>
            </div>
            <p class="text-sm text-gray-900 dark:text-white">
              {{ formatFileSize(selectedFile.size) }}
            </p>
          </div>

          <!-- Modified Date -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <Calendar :size="16" />
              <span class="text-xs font-medium uppercase">Modified</span>
            </div>
            <p class="text-sm text-gray-900 dark:text-white">
              {{ formatDate(selectedFile.modified) }}
            </p>
          </div>

          <!-- File Path -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <Info :size="16" />
              <span class="text-xs font-medium uppercase">Location</span>
            </div>
            <p class="text-xs text-gray-600 dark:text-gray-400 break-all font-mono bg-gray-50 dark:bg-gray-900 p-2 rounded">
              {{ selectedFile.path }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>
