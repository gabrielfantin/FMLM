<script setup lang="ts">
import { ref } from 'vue'
import FolderSidebar from './components/FolderSidebar.vue'
import ThumbnailGrid from './components/ThumbnailGrid.vue'
import MediaInfoPanel from './components/MediaInfoPanel.vue'
import { useMediaScanner } from './composables/useMediaScanner'
import { useWindowPersistence } from './composables/useWindowPersistence'
import { AlertCircle } from 'lucide-vue-next'

const { mediaFiles, isLoading, error, selectedFolderId, scanDirectory } = useMediaScanner()
const sidebarRef = ref<InstanceType<typeof FolderSidebar> | null>(null)
const infoPanelRef = ref<InstanceType<typeof MediaInfoPanel> | null>(null)
const isSidebarCollapsed = ref(false)
const isInfoPanelCollapsed = ref(false)
const selectedMediaFiles = ref<any[]>([])

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

function handleSelectionChange(files: any[]) {
  selectedMediaFiles.value = files
  console.log(`${files.length} file(s) selected`, files)
  // TODO: Implement actions for selected files (delete, move, etc.) in future phase
}

function toggleSidebar() {
  isSidebarCollapsed.value = !isSidebarCollapsed.value
}

function toggleInfoPanel() {
  isInfoPanelCollapsed.value = !isInfoPanelCollapsed.value
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
        <!-- Error Banner -->
        <div v-if="error" class="flex items-center gap-3 px-6 py-4 mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-800 dark:text-red-200 text-sm">
          <AlertCircle :size="24" class="flex-shrink-0 text-red-600 dark:text-red-400" />
          <span>{{ error }}</span>
        </div>

        <!-- Content -->
        <ThumbnailGrid 
          :media-files="mediaFiles"
          :is-sidebar-collapsed="isSidebarCollapsed"
          :is-info-panel-collapsed="isInfoPanelCollapsed"
          @selection-change="handleSelectionChange"
          @toggle-sidebar="toggleSidebar"
          @toggle-info-panel="toggleInfoPanel"
        />
      </div>
    </main>

    <!-- Media Info Panel -->
    <MediaInfoPanel 
      ref="infoPanelRef"
      :selected-file="selectedMediaFiles[0]"
      :is-collapsed="isInfoPanelCollapsed"
    />
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