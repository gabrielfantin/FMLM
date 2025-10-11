<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { FolderOpen, Folder, Trash2, Plus, Loader2 } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
import { useDatabase, type ScannedFolder } from '@/composables/useDatabase'

interface Props {
  selectedFolderId?: number | null
  isScanning?: boolean
  isCollapsed?: boolean
}

interface Emits {
  (e: 'folder-selected', folderId: number, path: string): void
  (e: 'new-folder-selected', path: string): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

const db = useDatabase()
const folders = ref<ScannedFolder[]>([])
const isLoadingFolders = ref(false)
const error = ref<string | null>(null)

// Load folders from database on mount
onMounted(async () => {
  await loadFolders()
})

async function loadFolders() {
  try {
    isLoadingFolders.value = true
    error.value = null
    folders.value = await db.getScannedFolders()
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err)
    console.error('Failed to load folders:', err)
  } finally {
    isLoadingFolders.value = false
  }
}

async function handleSelectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select a folder to scan',
    })

    if (selected && typeof selected === 'string') {
      emit('new-folder-selected', selected)
    }
  } catch (err) {
    console.error('Failed to select directory:', err)
  }
}

function handleFolderClick(folder: ScannedFolder) {
  emit('folder-selected', folder.id, folder.path)
}

async function handleForgetFolder(folder: ScannedFolder, event: Event) {
  event.stopPropagation()
  
  if (!confirm(`Remove "${folder.name}" from the list?`)) {
    return
  }

  try {
    await db.deleteScannedFolder(folder.id)
    await loadFolders()
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err)
    console.error('Failed to delete folder:', err)
  }
}

// Expose loadFolders so parent can refresh the list
defineExpose({
  loadFolders,
})
</script>

<template>
  <aside 
    :class="[
      'bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col h-screen transition-all duration-300',
      isCollapsed ? 'w-0 overflow-hidden' : 'w-80'
    ]"
  >
    <!-- Header -->
    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
      <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        Working Folders
      </h2>
      
      <!-- Add Folder Button -->
      <button
        @click="handleSelectDirectory"
        :disabled="isScanning"
        class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
      >
        <Plus v-if="!isScanning" :size="20" />
        <Loader2 v-else :size="20" class="animate-spin" />
        <span>{{ isScanning ? 'Scanning...' : 'Add Folder' }}</span>
      </button>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="p-4 bg-red-50 dark:bg-red-900/20 border-b border-red-200 dark:border-red-800">
      <p class="text-sm text-red-800 dark:text-red-200">{{ error }}</p>
    </div>

    <!-- Folder List -->
    <div class="flex-1 overflow-y-auto">
      <!-- Loading State -->
      <div v-if="isLoadingFolders" class="flex items-center justify-center py-12">
        <Loader2 :size="32" class="animate-spin text-indigo-600" />
      </div>

      <!-- Empty State -->
      <div v-else-if="folders.length === 0" class="flex flex-col items-center justify-center py-12 px-6 text-center">
        <FolderOpen :size="48" class="text-gray-400 mb-4" />
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
          No folders added yet
        </p>
        <p class="text-xs text-gray-500 dark:text-gray-500">
          Click "Add Folder" to get started
        </p>
      </div>

      <!-- Folder Items -->
      <div v-else class="py-2">
        <div
          v-for="folder in folders"
          :key="folder.id"
          @click="handleFolderClick(folder)"
          :class="[
            'w-full flex items-start gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group cursor-pointer',
            selectedFolderId === folder.id ? 'bg-indigo-50 dark:bg-indigo-900/20 border-l-4 border-indigo-600' : ''
          ]"
        >
          <Folder 
            :size="20" 
            :class="[
              'flex-shrink-0 mt-0.5',
              selectedFolderId === folder.id ? 'text-indigo-600 dark:text-indigo-400' : 'text-gray-400'
            ]"
          />
          
          <div class="flex-1 min-w-0">
            <p 
              :class="[
                'text-sm font-medium truncate',
                selectedFolderId === folder.id ? 'text-indigo-900 dark:text-indigo-100' : 'text-gray-900 dark:text-white'
              ]"
            >
              {{ folder.name }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5">
              {{ folder.path }}
            </p>
            <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
              {{ folder.file_count }} files
            </p>
          </div>

          <button
            @click="handleForgetFolder(folder, $event)"
            class="flex-shrink-0 p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-gray-400 hover:text-red-600 dark:hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all"
            title="Forget this folder"
          >
            <Trash2 :size="16" />
          </button>
        </div>
      </div>
    </div>

    <!-- Footer with App Title -->
    <div class="p-6 border-t border-gray-200 dark:border-gray-700">
      <h1 class="text-lg font-bold mb-1 bg-gradient-to-r from-indigo-500 to-purple-600 bg-clip-text text-transparent">
        FMLM
      </h1>
      <p class="text-xs text-gray-600 dark:text-gray-400 mb-3">
        File & Media Library Manager
      </p>
      <p class="text-xs text-gray-500 dark:text-gray-500">
        {{ folders.length }} folder{{ folders.length !== 1 ? 's' : '' }} tracked
      </p>
    </div>
  </aside>
</template>
