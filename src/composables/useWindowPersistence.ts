/**
 * Composable for window size persistence
 * 
 * Saves and restores window dimensions to/from the database
 */

import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { useDatabase } from './useDatabase'

const WINDOW_WIDTH_KEY = 'window_width'
const WINDOW_HEIGHT_KEY = 'window_height'

export function useWindowPersistence() {
  const db = useDatabase()
  const appWindow = getCurrentWindow()

  /**
   * Load and apply saved window size
   */
  async function restoreWindowSize() {
    try {
      const [widthStr, heightStr] = await Promise.all([
        db.getPreference(WINDOW_WIDTH_KEY),
        db.getPreference(WINDOW_HEIGHT_KEY),
      ])

      if (widthStr && heightStr) {
        const width = parseInt(widthStr, 10)
        const height = parseInt(heightStr, 10)

        if (width > 0 && height > 0) {
          const size = new LogicalSize(width, height)
          await appWindow.setSize(size)
          console.log(`Restored window size: ${width}x${height}`)
        }
      }
    } catch (err) {
      console.error('Failed to restore window size:', err)
    }
  }

  /**
   * Save current window size to database
   */
  async function saveWindowSize() {
    try {
      const size = await appWindow.innerSize()
      
      await Promise.all([
        db.setPreference(WINDOW_WIDTH_KEY, size.width.toString()),
        db.setPreference(WINDOW_HEIGHT_KEY, size.height.toString()),
      ])

      console.log(`Saved window size: ${size.width}x${size.height}`)
    } catch (err) {
      console.error('Failed to save window size:', err)
    }
  }

  /**
   * Handle window resize events with debouncing
   */
  let resizeTimeout: ReturnType<typeof setTimeout> | null = null
  async function handleResize() {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }

    // Debounce: save after 500ms of no resizing
    resizeTimeout = setTimeout(async () => {
      await saveWindowSize()
    }, 500)
  }

  // Set up event listeners
  let unlistenResize: (() => void) | null = null

  onMounted(async () => {
    // Restore window size on mount
    await restoreWindowSize()

    // Listen for resize events
    unlistenResize = await appWindow.onResized(handleResize)
  })

  // Clean up on unmount
  onUnmounted(async () => {
    if (unlistenResize) {
      unlistenResize()
    }
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    await saveWindowSize()
  })

  return {
    restoreWindowSize,
    saveWindowSize,
  }
}
