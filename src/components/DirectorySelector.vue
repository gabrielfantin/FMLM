<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, Check, Loader2 } from 'lucide-vue-next'

const emit = defineEmits<{
  directorySelected: [path: string]
}>()

defineProps<{
  isLoading?: boolean
}>()

const selectedPath = ref<string | null>(null)

async function selectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Media Folder',
    })

    if (selected && typeof selected === 'string') {
      selectedPath.value = selected
      emit('directorySelected', selected)
    }
  } catch (error) {
    console.error('Failed to open directory dialog:', error)
  }
}
</script>

<template>
  <div class="w-full p-8 bg-gradient-to-br from-indigo-500 via-purple-500 to-purple-600 rounded-xl shadow-lg">
    <div class="text-center mb-8 text-white">
      <h2 class="text-3xl font-semibold mb-2">Select Media Library</h2>
      <p class="text-sm opacity-90">Choose a folder containing your photos and videos</p>
    </div>

    <div class="flex flex-col items-center gap-4">
      <button 
        @click="selectDirectory" 
        :disabled="isLoading"
        class="flex items-center gap-3 px-8 py-4 text-lg font-semibold text-indigo-600 bg-white rounded-lg shadow-md transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5 active:translate-y-0 disabled:opacity-70 disabled:cursor-not-allowed"
      >
        <FolderOpen :size="24" v-if="!isLoading" />
        <Loader2 :size="24" class="animate-spin" v-else />
        <span v-if="!isLoading">{{ selectedPath ? 'Change Folder' : 'Choose Folder' }}</span>
        <span v-else>Scanning...</span>
      </button>

      <div v-if="selectedPath && !isLoading" class="flex items-center gap-2 px-5 py-3 bg-white/95 rounded-lg max-w-full overflow-hidden">
        <Check :size="20" class="text-green-500 flex-shrink-0" />
        <span class="text-gray-700 text-sm truncate">{{ selectedPath }}</span>
      </div>
    </div>
  </div>
</template>
