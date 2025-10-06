<script setup lang="ts">
import { ref } from 'vue'
import FolderSidebar from './components/FolderSidebar.vue'
import ThumbnailGrid from './components/ThumbnailGrid.vue'
import { useMediaScanner } from './composables/useMediaScanner'
import { AlertCircle } from 'lucide-vue-next'

const { mediaFiles, isLoading, error, selectedFolderId, scanDirectory } = useMediaScanner()
const sidebarRef = ref<InstanceType<typeof FolderSidebar> | null>(null)

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
</script>

<template>
  <div class="flex h-screen overflow-hidden">
    <!-- Left Sidebar -->
    <FolderSidebar 
      ref="sidebarRef"
      :selected-folder-id="selectedFolderId"
      :is-scanning="isLoading"
      @new-folder-selected="handleNewFolderSelected"
      @folder-selected="handleFolderSelected"
    />

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto">
      <div class="max-w-[1400px] mx-auto px-8 py-8 sm:px-4">
        <!-- Header -->
        <header class="text-center mb-8">
          <h1 class="text-4xl font-bold mb-2 bg-gradient-to-r from-indigo-500 to-purple-600 bg-clip-text text-transparent sm:text-3xl">
            FMLM - File & Media Library Manager
          </h1>
          <p class="text-lg text-gray-600 dark:text-gray-400">
            Organize and browse your photos and videos
          </p>
        </header>

        <!-- Error Banner -->
        <div v-if="error" class="flex items-center gap-3 px-6 py-4 mb-6 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-800 dark:text-red-200 text-sm">
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