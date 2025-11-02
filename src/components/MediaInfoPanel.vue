<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Image, Play, FileType, Calendar, HardDrive, Info, Loader2, Film, Music, Monitor, Clock } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { MediaFile } from '../composables/useMediaScanner'
import { useResizable } from '@/composables/useResizable'

interface VideoInfo {
  codec: string
  codec_long: string
  width: number
  height: number
  fps: number
  bitrate?: number
  pix_fmt: string
  aspect_ratio: string
}

interface AudioInfo {
  codec: string
  codec_long: string
  sample_rate: number
  channels: number
  bitrate?: number
  sample_fmt: string
}

interface GeneralInfo {
  format: string
  format_long: string
  duration?: number
  bitrate?: number
  size: number
}

interface MediaInfo {
  video?: VideoInfo
  audio?: AudioInfo
  general: GeneralInfo
  metadata: Record<string, string>
}

interface Props {
  selectedFile?: MediaFile | null
  isCollapsed?: boolean
}

const props = defineProps<Props>()

// Resizable panel
const { width: panelWidth, isResizing, startResize } = useResizable({
  defaultWidth: 384, // 96 * 4 = 384px (w-96)
  minWidth: 300,
  maxWidthPercent: 80, // 70% of screen width
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
const mediaInfo = ref<MediaInfo | null>(null)
const isLoadingInfo = ref(false)
const infoError = ref<string | null>(null)

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

// Format bitrate for display
function formatBitrate(bitrate: number): string {
  if (bitrate < 1000) return `${bitrate} bps`
  if (bitrate < 1000000) return `${(bitrate / 1000).toFixed(1)} Kbps`
  return `${(bitrate / 1000000).toFixed(2)} Mbps`
}

// Format duration for display
function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = Math.floor(seconds % 60)
  
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${minutes}:${String(secs).padStart(2, '0')}`
}

// Load detailed media information
async function loadMediaInfo() {
  if (!props.selectedFile) {
    mediaInfo.value = null
    return
  }

  isLoadingInfo.value = true
  infoError.value = null

  try {
    const info = await invoke<MediaInfo>('get_media_info', {
      filePath: props.selectedFile.path
    })
    mediaInfo.value = info
  } catch (error) {
    console.error('Failed to load media info:', error)
    infoError.value = error instanceof Error ? error.message : String(error)
    mediaInfo.value = null
  } finally {
    isLoadingInfo.value = false
  }
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
  loadMediaInfo()
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
          <!-- File Name (full width) -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <FileType :size="16" />
              <span class="text-xs font-medium uppercase">Name</span>
            </div>
            <p class="text-sm text-gray-900 dark:text-white break-all">
              {{ fileName }}
            </p>
          </div>

          <!-- 2-column grid for basic file info -->
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
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
            <div class="space-y-1 lg:col-span-2">
              <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                <Calendar :size="16" />
                <span class="text-xs font-medium uppercase">Modified</span>
              </div>
              <p class="text-sm text-gray-900 dark:text-white">
                {{ formatDate(selectedFile.modified) }}
              </p>
            </div>
          </div>

          <!-- File Path (full width) -->
          <div class="space-y-1">
            <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
              <Info :size="16" />
              <span class="text-xs font-medium uppercase">Location</span>
            </div>
            <p class="text-xs text-gray-600 dark:text-gray-400 break-all font-mono bg-gray-50 dark:bg-gray-900 p-2 rounded">
              {{ selectedFile.path }}
            </p>
          </div>

          <!-- Divider -->
          <div v-if="mediaInfo || isLoadingInfo" class="border-t border-gray-200 dark:border-gray-700"></div>

          <!-- Detailed Media Info Loading -->
          <div v-if="isLoadingInfo" class="flex items-center justify-center py-4">
            <Loader2 :size="24" class="animate-spin text-indigo-600" />
            <span class="ml-2 text-sm text-gray-600 dark:text-gray-400">Loading details...</span>
          </div>

          <!-- Detailed Media Info Error -->
          <div v-else-if="infoError" class="text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-2 rounded">
            Failed to load detailed info: {{ infoError }}
          </div>

          <!-- Detailed Media Info Content -->
          <template v-else-if="mediaInfo">
            <!-- Video Information -->
            <div v-if="mediaInfo.video" class="space-y-3">
              <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2 col-span-2">
                <Film :size="16" />
                Video Information
              </h3>

              <!-- 2-column grid for video info -->
              <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                <!-- Resolution -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <Monitor :size="16" />
                    <span class="text-xs font-medium uppercase">Resolution</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.video.width }} × {{ mediaInfo.video.height }}
                    <span class="text-xs text-gray-500 ml-1">({{ mediaInfo.video.aspect_ratio }})</span>
                  </p>
                </div>

                <!-- Frame Rate -->
                <div v-if="mediaInfo.video.fps > 0" class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <Film :size="16" />
                    <span class="text-xs font-medium uppercase">Frame Rate</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.video.fps.toFixed(2) }} fps
                  </p>
                </div>

                <!-- Codec -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <FileType :size="16" />
                    <span class="text-xs font-medium uppercase">Video Codec</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.video.codec.toUpperCase() }}
                  </p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    {{ mediaInfo.video.codec_long }}
                  </p>
                </div>

                <!-- Video Bitrate -->
                <div v-if="mediaInfo.video.bitrate" class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <HardDrive :size="16" />
                    <span class="text-xs font-medium uppercase">Video Bitrate</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ formatBitrate(mediaInfo.video.bitrate) }}
                  </p>
                </div>

                <!-- Pixel Format -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <Info :size="16" />
                    <span class="text-xs font-medium uppercase">Pixel Format</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.video.pix_fmt }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Audio Information -->
            <div v-if="mediaInfo.audio" class="space-y-3">
              <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2 col-span-2">
                <Music :size="16" />
                Audio Information
              </h3>

              <!-- 2-column grid for audio info -->
              <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                <!-- Audio Codec -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <FileType :size="16" />
                    <span class="text-xs font-medium uppercase">Audio Codec</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.audio.codec.toUpperCase() }}
                  </p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    {{ mediaInfo.audio.codec_long }}
                  </p>
                </div>

                <!-- Sample Rate -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <Music :size="16" />
                    <span class="text-xs font-medium uppercase">Sample Rate</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ (mediaInfo.audio.sample_rate / 1000).toFixed(1) }} kHz
                  </p>
                </div>

                <!-- Channels -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <Music :size="16" />
                    <span class="text-xs font-medium uppercase">Channels</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.audio.channels }} {{ mediaInfo.audio.channels === 1 ? 'channel' : 'channels' }}
                    <span v-if="mediaInfo.audio.channels === 2" class="text-xs text-gray-500">(Stereo)</span>
                    <span v-else-if="mediaInfo.audio.channels === 1" class="text-xs text-gray-500">(Mono)</span>
                    <span v-else-if="mediaInfo.audio.channels > 2" class="text-xs text-gray-500">(Surround)</span>
                  </p>
                </div>

                <!-- Audio Bitrate -->
                <div v-if="mediaInfo.audio.bitrate" class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <HardDrive :size="16" />
                    <span class="text-xs font-medium uppercase">Audio Bitrate</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ formatBitrate(mediaInfo.audio.bitrate) }}
                  </p>
                </div>
              </div>
            </div>

            <!-- General Information -->
            <div class="space-y-3">
              <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2 col-span-2">
                <Info :size="16" />
                General Information
              </h3>

              <!-- 2-column grid for general info -->
              <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                <!-- Format -->
                <div class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <FileType :size="16" />
                    <span class="text-xs font-medium uppercase">Format</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ mediaInfo.general.format }}
                  </p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    {{ mediaInfo.general.format_long }}
                  </p>
                </div>

                <!-- Duration -->
                <div v-if="mediaInfo.general.duration" class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <Clock :size="16" />
                    <span class="text-xs font-medium uppercase">Duration</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ formatDuration(mediaInfo.general.duration) }}
                  </p>
                </div>

                <!-- Overall Bitrate -->
                <div v-if="mediaInfo.general.bitrate" class="space-y-1">
                  <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                    <HardDrive :size="16" />
                    <span class="text-xs font-medium uppercase">Overall Bitrate</span>
                  </div>
                  <p class="text-sm text-gray-900 dark:text-white">
                    {{ formatBitrate(mediaInfo.general.bitrate) }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Additional Metadata -->
            <div v-if="Object.keys(mediaInfo.metadata).length > 0" class="space-y-2">
              <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2">
                <Info :size="16" />
                Metadata
              </h3>
              <div class="text-xs space-y-1 max-h-48 overflow-y-auto bg-gray-50 dark:bg-gray-900 p-2 rounded">
                <div v-for="(value, key) in mediaInfo.metadata" :key="key" class="flex gap-2">
                  <span class="text-gray-500 dark:text-gray-400 font-medium">{{ key }}:</span>
                  <span class="text-gray-900 dark:text-white flex-1 break-all">{{ value }}</span>
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>
  </aside>
</template>
