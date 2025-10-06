<script setup lang="ts">
import { computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Image, Play } from 'lucide-vue-next'
import type { MediaFile } from '../composables/useMediaScanner'

const props = defineProps<{
  mediaFiles: MediaFile[]
}>()

const emit = defineEmits<{
  fileClick: [file: MediaFile]
}>()

// Convert file paths to URLs that Tauri can serve
const mediaItems = computed(() => {
  return props.mediaFiles.map(file => ({
    ...file,
    url: convertFileSrc(file.path),
  }))
})

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
  return date.toLocaleDateString()
}

function handleFileClick(file: MediaFile) {
  emit('fileClick', file)
}
</script>

<template>
  <div class="w-full min-h-[400px]">
    <!-- Empty State -->
    <div v-if="mediaFiles.length === 0" class="flex flex-col items-center justify-center py-16 px-8 text-gray-400">
      <Image :size="80" class="mb-6 opacity-50" />
      <p class="text-xl font-semibold text-gray-600 mb-2">No media files found</p>
      <p class="text-sm">Select a folder containing photos or videos</p>
    </div>

    <!-- Thumbnail Grid -->
    <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-3 sm:grid-cols-[repeat(auto-fill,minmax(150px,1fr))] sm:gap-2 lg:grid-cols-[repeat(auto-fill,minmax(200px,1fr))] lg:gap-3 2xl:grid-cols-[repeat(auto-fill,minmax(220px,1fr))]">
      <div 
        v-for="item in mediaItems" 
        :key="item.path"
        class="relative cursor-pointer transition-transform duration-200 hover:-translate-y-1"
        @click="handleFileClick(item)"
      >
        <!-- Thumbnail Wrapper -->
        <div class="relative bg-gray-100 rounded-lg overflow-hidden shadow-sm transition-shadow hover:shadow-lg">
          <!-- Image Thumbnail -->
          <img 
            v-if="item.media_type === 'image'"
            :src="item.url" 
            :alt="item.name"
            class="w-full h-[180px] object-cover block sm:h-[140px]"
            loading="lazy"
          />
          
          <!-- Video Thumbnail -->
          <div v-else-if="item.media_type === 'video'" class="relative">
            <video 
              :src="item.url"
              class="w-full h-[180px] object-cover block sm:h-[140px]"
              preload="metadata"
            />
            <div class="absolute inset-0 flex items-center justify-center bg-black/30 pointer-events-none">
              <Play :size="40" class="text-white drop-shadow-lg" fill="currentColor" />
            </div>
          </div>

          <!-- File Info -->
          <div class="p-2 bg-white">
            <span class="block text-xs font-medium text-gray-800 truncate mb-0.5" :title="item.name">
              {{ item.name }}
            </span>
            <div class="flex justify-between items-center text-[0.65rem] text-gray-500">
              <span>{{ formatFileSize(item.size) }}</span>
              <span>{{ formatDate(item.modified) }}</span>
            </div>
          </div>
        </div>

        <!-- Type Badge -->
        <div 
          class="absolute top-1.5 right-1.5 px-1.5 py-0.5 text-[0.65rem] font-semibold text-white rounded backdrop-blur-sm shadow-md"
          :class="{
            'bg-blue-500/90': item.media_type === 'image',
            'bg-red-500/90': item.media_type === 'video',
            'bg-gray-500/90': item.media_type === 'unknown'
          }"
        >
          {{ item.file_type.toUpperCase() }}
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div v-if="mediaFiles.length > 0" class="mt-4 p-3 text-center text-gray-500 text-sm">
      <p class="font-medium">
        {{ mediaFiles.length }} {{ mediaFiles.length === 1 ? 'file' : 'files' }} found
      </p>
    </div>
  </div>
</template>
