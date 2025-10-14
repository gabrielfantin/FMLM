import { ref, onMounted, onUnmounted, computed } from 'vue'

interface UseResizableOptions {
  minWidth?: number
  maxWidth?: number
  maxWidthPercent?: number  // Maximum width as percentage of window width
  defaultWidth: number
  side: 'left' | 'right'
}

/**
 * Composable for making panels resizable by dragging their edge
 */
export function useResizable(options: UseResizableOptions) {
  const { minWidth = 200, maxWidth, maxWidthPercent, defaultWidth, side } = options
  
  const width = ref(defaultWidth)
  const isResizing = ref(false)
  
  let startX = 0
  let startWidth = 0
  
  // Calculate effective max width (use percentage if specified, otherwise use fixed value)
  const effectiveMaxWidth = computed(() => {
    if (maxWidthPercent) {
      return Math.floor(window.innerWidth * (maxWidthPercent / 100))
    }
    return maxWidth || 800
  })
  
  function startResize(event: MouseEvent) {
    isResizing.value = true
    startX = event.clientX
    startWidth = width.value
    
    // Prevent text selection during drag
    document.body.style.userSelect = 'none'
    document.body.style.cursor = 'col-resize'
    
    event.preventDefault()
  }
  
  function resize(event: MouseEvent) {
    if (!isResizing.value) return
    
    const delta = side === 'left' 
      ? event.clientX - startX 
      : startX - event.clientX
    
    const newWidth = startWidth + delta
    
    // Clamp width between min and max
    width.value = Math.max(minWidth, Math.min(effectiveMaxWidth.value, newWidth))
  }
  
  function stopResize() {
    if (!isResizing.value) return
    
    isResizing.value = false
    document.body.style.userSelect = ''
    document.body.style.cursor = ''
  }
  
  onMounted(() => {
    document.addEventListener('mousemove', resize)
    document.addEventListener('mouseup', stopResize)
  })
  
  onUnmounted(() => {
    document.removeEventListener('mousemove', resize)
    document.removeEventListener('mouseup', stopResize)
  })
  
  return {
    width,
    isResizing,
    startResize,
  }
}
