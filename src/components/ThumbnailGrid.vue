<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { Image, Play, Grid3x3, Grid2x2, LayoutGrid } from 'lucide-vue-next'
import type { MediaFile } from '../composables/useMediaScanner'
import { useThumbnails } from '../composables/useThumbnails'

const props = defineProps<{
  mediaFiles: MediaFile[]
}>()

const emit = defineEmits<{
  selectionChange: [selectedFiles: MediaFile[]]
}>()

type CardSize = 'small' | 'medium' | 'large'

const { generateThumbnail, getThumbnailPath } = useThumbnails()
const thumbnailUrls = ref<Map<string, string>>(new Map())
const loadingThumbnails = ref<Set<string>>(new Set())
const cardSize = ref<CardSize>('medium')

// Selection state
const selectedPaths = ref<Set<string>>(new Set())
const lastSelectedIndex = ref<number>(-1)
const focusedIndex = ref<number>(-1)

// Card size configurations
const cardSizeConfig = computed(() => {
  const configs = {
    small: {
      minWidth: '120px',
      height: '120px',
    },
    medium: {
      minWidth: '160px',
      height: '160px',
    },
    large: {
      minWidth: '220px',
      height: '220px',
    },
  }
  return configs[cardSize.value]
})

// Create computed items with thumbnail URLs
const mediaItems = computed(() => {
  return props.mediaFiles.map((file, index) => ({
    ...file,
    thumbnailUrl: thumbnailUrls.value.get(file.path),
    isLoadingThumbnail: loadingThumbnails.value.has(file.path),
    isSelected: selectedPaths.value.has(file.path),
    isFocused: focusedIndex.value === index,
    index,
  }))
})

// Get selected files
const selectedFiles = computed(() => {
  return props.mediaFiles.filter(file => selectedPaths.value.has(file.path))
})

// Load thumbnails for all media files
async function loadThumbnails() {
  for (const file of props.mediaFiles) {
    // Skip if already loaded
    if (thumbnailUrls.value.has(file.path)) {
      continue
    }

    // Check if thumbnail exists in cache
    const cachedThumbnail = await getThumbnailPath(file.path)
    if (cachedThumbnail) {
      thumbnailUrls.value.set(file.path, cachedThumbnail)
      continue
    }

    // Generate thumbnail in background
    loadingThumbnails.value.add(file.path)
    const isVideo = file.media_type === 'video'
    
    generateThumbnail(file.path, isVideo).then(thumbnailUrl => {
      loadingThumbnails.value.delete(file.path)
      if (thumbnailUrl) {
        thumbnailUrls.value.set(file.path, thumbnailUrl)
      }
    }).catch(error => {
      console.error('Failed to generate thumbnail for', file.path, error)
      loadingThumbnails.value.delete(file.path)
    })
  }
}

// Watch for changes in media files
watch(() => props.mediaFiles, () => {
  loadThumbnails()
}, { immediate: true })

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

// Selection handlers
function handleCardClick(item: typeof mediaItems.value[0], event: MouseEvent) {
  const index = item.index

  if (event.ctrlKey || event.metaKey) {
    // Toggle selection with Ctrl/Cmd
    if (selectedPaths.value.has(item.path)) {
      selectedPaths.value.delete(item.path)
    } else {
      selectedPaths.value.add(item.path)
    }
    lastSelectedIndex.value = index
  } else if (event.shiftKey && lastSelectedIndex.value >= 0) {
    // Range selection with Shift
    const start = Math.min(lastSelectedIndex.value, index)
    const end = Math.max(lastSelectedIndex.value, index)
    for (let i = start; i <= end; i++) {
      if (props.mediaFiles[i]) {
        selectedPaths.value.add(props.mediaFiles[i].path)
      }
    }
  } else {
    // Single selection
    selectedPaths.value.clear()
    selectedPaths.value.add(item.path)
    lastSelectedIndex.value = index
  }

  focusedIndex.value = index
  emit('selectionChange', selectedFiles.value)
}

// Keyboard navigation
function handleKeyDown(event: KeyboardEvent) {
  if (props.mediaFiles.length === 0) return

  const currentIndex = focusedIndex.value >= 0 ? focusedIndex.value : 0

  switch (event.key) {
    case 'Escape':
      selectedPaths.value.clear()
      focusedIndex.value = -1
      emit('selectionChange', [])
      break

    case 'ArrowRight':
    case 'ArrowDown': {
      event.preventDefault()
      const nextIndex = Math.min(currentIndex + 1, props.mediaFiles.length - 1)
      focusedIndex.value = nextIndex

      if (event.shiftKey) {
        selectedPaths.value.add(props.mediaFiles[nextIndex].path)
      } else if (!event.ctrlKey && !event.metaKey) {
        selectedPaths.value.clear()
        selectedPaths.value.add(props.mediaFiles[nextIndex].path)
        lastSelectedIndex.value = nextIndex
      }
      emit('selectionChange', selectedFiles.value)
      scrollToIndex(nextIndex)
      break
    }

    case 'ArrowLeft':
    case 'ArrowUp': {
      event.preventDefault()
      const prevIndex = Math.max(currentIndex - 1, 0)
      focusedIndex.value = prevIndex

      if (event.shiftKey) {
        selectedPaths.value.add(props.mediaFiles[prevIndex].path)
      } else if (!event.ctrlKey && !event.metaKey) {
        selectedPaths.value.clear()
        selectedPaths.value.add(props.mediaFiles[prevIndex].path)
        lastSelectedIndex.value = prevIndex
      }
      emit('selectionChange', selectedFiles.value)
      scrollToIndex(prevIndex)
      break
    }

    case 'a':
    case 'A':
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault()
        props.mediaFiles.forEach(file => selectedPaths.value.add(file.path))
        emit('selectionChange', selectedFiles.value)
      }
      break
  }
}

// Scroll focused item into view
function scrollToIndex(index: number) {
  const element = document.querySelector(`[data-index="${index}"]`)
  if (element) {
    element.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }
}

// Setup keyboard listeners
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
  loadThumbnails()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <div class="w-full min-h-[400px]">
    <!-- Card Size Controls -->
    <div v-if="mediaFiles.length > 0" class="fixed top-4 right-4 z-40 flex items-center gap-1 p-1.5 bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700">
      <button
        @click="cardSize = 'small'"
        :class="[
          'p-2 rounded-md transition-colors',
          cardSize === 'small'
            ? 'bg-indigo-600 text-white shadow-sm'
            : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
        ]"
        title="Small cards"
      >
        <Grid3x3 :size="18" />
      </button>
      <button
        @click="cardSize = 'medium'"
        :class="[
          'p-2 rounded-md transition-colors',
          cardSize === 'medium'
            ? 'bg-indigo-600 text-white shadow-sm'
            : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
        ]"
        title="Medium cards"
      >
        <Grid2x2 :size="18" />
      </button>
      <button
        @click="cardSize = 'large'"
        :class="[
          'p-2 rounded-md transition-colors',
          cardSize === 'large'
            ? 'bg-indigo-600 text-white shadow-sm'
            : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
        ]"
        title="Large cards"
      >
        <LayoutGrid :size="18" />
      </button>
    </div>

    <!-- Empty State -->
    <div v-if="mediaFiles.length === 0" class="flex flex-col items-center justify-center py-16 px-8 text-gray-400">
      <Image :size="80" class="mb-6 opacity-50" />
      <p class="text-xl font-semibold text-gray-600 mb-2">No media files found</p>
      <p class="text-sm">Select a folder containing photos or videos</p>
    </div>

    <!-- Thumbnail Grid -->
    <div v-else class="grid gap-1" :style="{ gridTemplateColumns: `repeat(auto-fill, minmax(${cardSizeConfig.minWidth}, 1fr))` }">
      <div 
        v-for="item in mediaItems" 
        :key="item.path"
        :data-index="item.index"
        :class="[
          'relative cursor-pointer transition-all duration-200',
          item.isSelected ? 'scale-110 z-30' : 'hover:-translate-y-1',
          item.isFocused && !item.isSelected ? 'ring-2 ring-indigo-400 ring-offset-2' : ''
        ]"
        @click="handleCardClick(item, $event)"
      >
        <!-- Thumbnail Wrapper -->
        <div 
          :class="[
            'relative bg-gray-100 rounded overflow-hidden transition-shadow',
            item.isSelected 
              ? 'shadow-2xl ring-4 ring-indigo-600' 
              : 'shadow-sm hover:shadow-lg'
          ]"
        >
          <!-- Thumbnail (for both images and videos) -->
          <div v-if="item.thumbnailUrl" class="relative">
            <img 
              :src="item.thumbnailUrl" 
              :alt="item.name"
              class="w-full object-cover block"
              :style="{ height: cardSizeConfig.height }"
              loading="lazy"
            />
            <!-- Video overlay -->
            <div v-if="item.media_type === 'video'" class="absolute inset-0 flex items-center justify-center bg-black/30 pointer-events-none">
              <Play :size="40" class="text-white drop-shadow-lg" fill="currentColor" />
            </div>
          </div>
          
          <!-- Loading state -->
          <div v-else-if="item.isLoadingThumbnail" class="w-full flex items-center justify-center bg-gray-200" :style="{ height: cardSizeConfig.height }">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-600"></div>
          </div>
          
          <!-- Fallback: Placeholder while thumbnail hasn't loaded yet -->
          <div v-else class="w-full flex items-center justify-center bg-gray-200" :style="{ height: cardSizeConfig.height }">
            <Image v-if="item.media_type === 'image'" :size="32" class="text-gray-400" />
            <Play v-else :size="32" class="text-gray-400" />
          </div>

          <!-- File Info -->
          <div class="p-1.5 bg-white">
            <span class="block text-[0.65rem] font-medium text-gray-800 truncate" :title="item.name">
              {{ item.name }}
            </span>
            <div class="flex justify-between items-center text-[0.6rem] text-gray-500 mt-0.5">
              <span>{{ formatFileSize(item.size) }}</span>
              <span>{{ formatDate(item.modified) }}</span>
            </div>
          </div>
        </div>

        <!-- Type Badge -->
        <div 
          class="absolute top-1 right-1 px-1 py-0.5 text-[0.6rem] font-semibold text-white rounded backdrop-blur-sm shadow-md"
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
    <div v-if="mediaFiles.length > 0" class="mt-2 p-2 text-center text-gray-500 text-sm">
      <p class="font-medium">
        {{ mediaFiles.length }} {{ mediaFiles.length === 1 ? 'file' : 'files' }} found
      </p>
    </div>
  </div>
</template>
