<script setup lang="ts">
import DirectorySelector from './components/DirectorySelector.vue'
import ThumbnailGrid from './components/ThumbnailGrid.vue'
import { useMediaScanner } from './composables/useMediaScanner'
import { AlertCircle } from 'lucide-vue-next'

const { mediaFiles, isLoading, error, scanDirectory } = useMediaScanner()

async function handleDirectorySelected(path: string) {
  await scanDirectory(path, true)
}

function handleFileClick(file: any) {
  console.log('File clicked:', file)
  // TODO: Implement file viewer/slideshow in future phase
}
</script>

<template>
  <main class="max-w-[1400px] mx-auto px-8 py-8 min-h-screen sm:px-4">
    <!-- Header -->
    <header class="text-center mb-8">
      <h1 class="text-4xl font-bold mb-2 bg-gradient-to-r from-indigo-500 to-purple-600 bg-clip-text text-transparent sm:text-3xl">
        FMLM - File & Media Library Manager
      </h1>
      <p class="text-lg text-gray-600 dark:text-gray-400">
        Organize and browse your photos and videos
      </p>
    </header>

    <!-- Content -->
    <div class="flex flex-col gap-8">
      <DirectorySelector 
        :is-loading="isLoading"
        @directory-selected="handleDirectorySelected"
      />

      <!-- Error Banner -->
      <div v-if="error" class="flex items-center gap-3 px-6 py-4 my-6 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
        <AlertCircle :size="24" class="flex-shrink-0 text-red-600" />
        <span>{{ error }}</span>
      </div>

      <ThumbnailGrid 
        :media-files="mediaFiles"
        @file-click="handleFileClick"
      />
    </div>
  </main>
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