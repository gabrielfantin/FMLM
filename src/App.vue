<script setup lang="ts">
import { ref } from 'vue'
import FolderSidebar from './components/FolderSidebar.vue'
import ThumbnailGrid from './components/ThumbnailGrid.vue'
import { useMediaScanner } from './composables/useMediaScanner'
import { useWindowPersistence } from './composables/useWindowPersistence'
import { AlertCircle, PanelLeftClose, PanelLeft } from 'lucide-vue-next'

const { mediaFiles, isLoading, error, selectedFolderId, scanDirectory } = useMediaScanner()
const sidebarRef = ref<InstanceType<typeof FolderSidebar> | null>(null)
const isSidebarCollapsed = ref(false)

// Initialize window size persistence
useWindowPersistence()

async function handleNewFolderSelected(path: string) {
  await scanDirectory(path, true)
  // Refresh the sidebar folder list after scanning
  sidebarRef.value?.loadFolders()
}

async function handleFolderSelected(_folderId: number, path: string) {
  await scanDirectory(path, true)
}

function handleFileClick(file: any) {
  console.log('File clicked:', file)
  // TODO: Implement file viewer/slideshow in future phase
}

function toggleSidebar() {
  isSidebarCollapsed.value = !isSidebarCollapsed.value
}
</script>

<template>
  <div class="flex h-screen overflow-hidden">
    <!-- Left Sidebar -->
    <FolderSidebar 
      ref="sidebarRef"
      :selected-folder-id="selectedFolderId"
      :is-scanning="isLoading"
      :is-collapsed="isSidebarCollapsed"
      @new-folder-selected="handleNewFolderSelected"
      @folder-selected="handleFolderSelected"
    />

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto">
      <div class="h-full px-4 py-4">
        <!-- Toggle Sidebar Button -->
        <button
          @click="toggleSidebar"
          class="mb-4 flex items-center gap-2 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors shadow-sm"
          :title="isSidebarCollapsed ? 'Show sidebar' : 'Hide sidebar'"
        >
          <PanelLeft v-if="isSidebarCollapsed" :size="20" class="text-gray-600 dark:text-gray-400" />
          <PanelLeftClose v-else :size="20" class="text-gray-600 dark:text-gray-400" />
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
            {{ isSidebarCollapsed ? 'Show' : 'Hide' }} Sidebar
          </span>
        </button>
        <!-- Error Banner -->
        <div v-if="error" class="flex items-center gap-3 px-6 py-4 mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-800 dark:text-red-200 text-sm">
          <AlertCircle :size="24" class="flex-shrink-0 text-red-600 dark:text-red-400" />
          <span>{{ error }}</span>
        </div>

        <!-- Content -->
        <ThumbnailGrid 
          :media-files="mediaFiles"
          @file-click="handleFileClick"
        />
      </div>
    </main>
  </div>
</template>

<style>
:root {
  font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  padding: 0;
  min-height: 100vh;
  background-color: #f9fafb;
}

#app {
  min-height: 100vh;
}

@media (prefers-color-scheme: dark) {
  body {
    background-color: #1f2937;
  }
}
</style>