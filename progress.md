# FMLM Progress Documentation

This document tracks the status of features planned and implemented in the FMLM app. Each feature is listed with its current implementation status.

## Project Setup

**Package Manager:** Yarn (v1.22.22)
- ✅ Migrated from npm to Yarn
- ✅ Dependencies installed and working
- ✅ Dev server running successfully with `yarn tauri dev`

**Styling & Icons:**
- ✅ Tailwind CSS v4.1 configured with @tailwindcss/vite plugin
- ✅ Lucide Vue Next for icon components
- ✅ CSS imported in src/style.css with `@import "tailwindcss"`
- ✅ All components refactored to use Tailwind utilities (removed ~400+ lines of custom CSS)
- ✅ All SVG icons replaced with Lucide components (FolderOpen, Check, Loader2, Image, Play, AlertCircle)

**Rust Backend:**
- ✅ Code reviewed and updated to follow Rust best practices and guidelines
- ✅ Comprehensive documentation added (module, function, and example docs)
- ✅ Expanded test coverage from 1 to 6 unit tests + 1 doctest
- ✅ Added common trait implementations (PartialEq, Eq, Copy, Default)
- ✅ Extracted helper functions for better code organization
- ✅ Passes all quality checks: `cargo check`, `cargo test`, `cargo clippy` (zero warnings)
- ✅ Updated Cargo.toml with proper metadata (license, repository, keywords, categories)

**Database:**
- ✅ SQLite integration with sqlx for async operations
- ✅ Database stored in user's system Tauri app data directory
- ✅ Automatic migrations on app startup
- ✅ Three main tables: scanned_folders, media_metadata, user_preferences
- ✅ Full CRUD operations via Tauri commands
- ✅ Connection pooling for performance
- ✅ Foreign key constraints and indexes for data integrity
- ✅ Stores media metadata (dimensions, duration, file info, thumbnails)
- ✅ Stores user preferences and settings
- ✅ Tracks scanned folders with last scan time and file counts

**Commands:**
- `yarn install` - Install dependencies
- `yarn dev` - Start Vite dev server only
- `yarn tauri dev` - Start full Tauri app in dev mode
- `yarn build` - Build for production

## Phase 1: Core Viewing and Browsing

### Thumbnail Grid View (FR-3.2): In Progress (Base Implementation Complete)

**Implemented:**
- ✅ Directory selection with native dialog (Tauri plugin-dialog)
- ✅ Recursive directory scanning for image and video files
- ✅ Support for multiple image formats (jpg, jpeg, png, gif, bmp, webp, heic, heif, tiff, svg)
- ✅ Support for multiple video formats (mp4, mov, avi, mkv, webm, flv, wmv, m4v, mpg, mpeg)
- ✅ Responsive CSS Grid layout (1-2 cols mobile, 4-6 cols desktop)
- ✅ File metadata extraction (size, modified date, file type)
- ✅ Direct file display using Tauri's convertFileSrc
- ✅ Video preview with play icon overlay
- ✅ File type badges (Image/Video)
- ✅ Empty state messaging
- ✅ Loading states with spinner
- ✅ Error handling and display
- ✅ **Working Folders Sidebar:**
  - Left sidebar menu displaying tracked folders
  - "Add Folder" button to scan new directories
  - Folder list with name, path, and file count
  - Click to load folder contents
  - "Forget" button to remove folders from tracking
  - Folders automatically saved to SQLite database
  - Visual indication of currently selected folder
  - Persistent folder history across app sessions
  - App title and branding in sidebar footer
- ✅ **UI/UX Improvements:**
  - Compact thumbnail grid layout for better space utilization
  - Reduced spacing between items (gap: 3px on desktop, 2px on mobile)
  - Smaller thumbnails (180px default, 140px mobile) to fit more items
  - Reduced padding in file info cards
  - More content visible without scrolling
  - **Collapsible sidebar** with toggle button
  - **Minimal gap grid** layout (1px gap) for maximum space efficiency
  - **Adaptive columns** using CSS Grid auto-fill (minmax(160px, 1fr))
  - Grid automatically adjusts number of columns based on available width
  - Smooth sidebar collapse animation (300ms transition)
  - Toggle button with visual feedback (Show/Hide Sidebar)
  - **Card size selector** with three options: Small (120px), Medium (160px), Large (220px)
  - Dynamic grid that adapts to selected card size
  - Persistent size selection during session
  - Visual indication of currently selected size
  - **Compact icon-only controls** for minimal UI footprint
  - Icon-only sidebar toggle button positioned at sidebar border (fixed, left side)
  - Icon-only size selector (Grid3x3, Grid2x2, LayoutGrid icons)
  - Size controls positioned fixed at top-right corner
  - Reduced control bar padding and spacing
  - Controls float above content without affecting layout
- ✅ **Window State Persistence:**
  - Window size automatically saved to database on resize
  - Window size restored on app startup
  - Debounced save (500ms) to avoid excessive database writes
  - Default window size: 1200x800 (min: 800x600)
- ✅ **Thumbnail Generation & Caching:**
  - Automatic thumbnail generation for images using the `image` crate
  - Video thumbnail extraction using `rsmpeg` with FFmpeg 8.0 support (captures frame at 10% into video)
  - Thumbnails cached in user's system cache directory (256x256 JPEG, quality 85)
  - SHA-256 hash-based cache keys to avoid regenerating thumbnails
  - Background thumbnail generation with concurrency control (max 5 simultaneous)
  - Base64 data URLs for browser compatibility (no asset protocol restrictions)
  - Comprehensive structured logging with `tracing` for debugging
  - Thumbnails displayed as they become available with loading indicators
  - Significantly improved performance and reduced memory usage
  - Tauri commands: generate_thumbnail, thumbnail_exists, get_thumbnail_path, clear_thumbnail_cache, get_cache_size, generate_thumbnails_batch
  - Dependencies: `rsmpeg`, `base64`, `tracing`, `tracing-subscriber`, `once_cell`, `futures`
- ✅ **Media Selection System:**
  - Single-click selection with visual feedback
  - **Selected cards are enlarged** (scale 1.1) with indigo ring border
  - **Multiple selection support:**
    - Ctrl/Cmd + Click: Toggle individual items
    - Shift + Click: Range selection from last selected item
    - Ctrl/Cmd + A: Select all items
  - **Keyboard navigation:**
    - Arrow keys (Left/Right) to navigate to previous/next item
    - Arrow keys (Up/Down) to navigate to row above/below (dynamic grid-aware)
    - Arrow keys with Shift to extend selection
    - Arrow keys with Ctrl/Cmd to move focus without changing selection
    - Esc key to clear all selections
  - Focused item indicator (ring outline) separate from selection
  - Smooth scrolling to bring focused item into view
  - Selection state management with reactive updates
  - Parent component receives selection change events
- ✅ **Media Info Panel:**
  - Right sidebar panel showing detailed information about selected media
  - **Full media preview** with original file loaded:
    - Images displayed using Tauri's asset protocol with convertFileSrc
    - Videos with HTML5 video player controls and streaming support
    - **Streaming support** for large video files using Tauri's asset protocol (supports HTTP range requests)
    - No memory issues - files are streamed on demand instead of loaded entirely
  - **Comprehensive file details:**
    - File name with proper formatting
    - File type with color-coded badge (Image/Video)
    - File size with human-readable formatting (B, KB, MB, GB)
    - Last modified date and time with locale formatting
    - Full file path in monospace font
  - **Collapsible panel** with toggle button (similar to folder sidebar)
  - Shows info only for first selected file when multiple are selected
  - Empty state when no file is selected
  - Loading indicator during media file load
  - Error handling for failed file access
  - **Backend support:**
    - New Rust commands: `get_asset_url`, `get_mime_type`
    - Secure file access through Tauri's asset protocol
    - Asset protocol enabled with full scope for file access
    - Support for all image and video MIME types
  - **Icon-only toggle button** that sticks with the panel border:
    - Dynamically positioned based on actual panel width
    - Follows panel edge when resized
    - Fixed positioning at top-right
  - **Resizable panel** with draggable left edge:
    - Drag the left border to resize between 300px and up to 70% of screen width
    - Custom `useResizable` composable for smooth drag-to-resize
    - Visual feedback on hover and during resize (indigo highlight)
    - Width automatically adapts to user preference and screen size
  - **Compact layout** for maximum media preview space:
    - Removed "Media Info" header
    - Removed "Preview" section title
    - Reduced padding from 24px to 12px (p-6 to p-3)
    - Reduced spacing between elements from 24px to 12px (space-y-6 to space-y-3)
  - **Enhanced Media Metadata Display:**
    - **Comprehensive video information:**
      - Video codec name and description (e.g., "H264 - H.264 / AVC / MPEG-4 AVC")
      - Resolution with aspect ratio (e.g., "1920 × 1080 (16:9)")
      - Frame rate in fps
      - Video bitrate (formatted as Mbps/Kbps)
      - Pixel format (e.g., "yuv420p")
    - **Detailed audio information:**
      - Audio codec name and description (e.g., "AAC - AAC (Advanced Audio Coding)")
      - Sample rate in kHz
      - Number of channels (Mono/Stereo/Surround)
      - Audio bitrate
      - Sample format
    - **General file information:**
      - Container format name and description
      - Duration (formatted as HH:MM:SS or MM:SS)
      - Overall bitrate
      - File size
    - **Additional metadata** extracted from file (tags, EXIF, etc.)
    - **Image information:**
      - Image dimensions
      - Pixel format/color space
      - Format-specific details
    - **On-demand metadata extraction:**
      - Metadata loaded when file is selected (not during directory scan)
      - Loading indicator while extracting information
      - Error handling for unsupported formats
    - **Backend implementation:**
      - New `media_info.rs` module using FFmpeg for comprehensive metadata extraction
      - Separate handling for images (using `image` crate) and videos (using `rsmpeg`)
      - Codec information extracted via FFmpeg's avcodec_descriptor API
      - Database schema extended with metadata fields:
        - video_codec, video_codec_long
        - audio_codec, audio_codec_long
        - bitrate, frame_rate
        - sample_rate, audio_channels
        - format, metadata_json
      - Optional database caching for future performance improvements
    - **Dependencies:** `rsmpeg`, `image` crate with GenericImageView trait
- ✅ **Cleaner Thumbnail Cards:**
  - Removed file info panel from individual cards
  - Cards now show only thumbnails and type badges
  - More space-efficient layout
  - Info moved to dedicated MediaInfoPanel for focused viewing
- ✅ **Resizable Sidebars:**
  - **Left sidebar (Working Folders)** is resizable:
    - Drag the right border to resize between 200px and 600px
    - Default width: 320px (w-80)
    - Toggle button dynamically follows sidebar edge
  - **Right panel (Media Info)** is resizable:
    - Drag the left border to resize between 300px and 70% of screen width
    - Default width: 384px (w-96)
    - Toggle button dynamically follows panel edge
  - **Custom resize implementation:**
    - No external dependencies - built with vanilla JavaScript
    - `useResizable` composable handles all resize logic
    - Supports both fixed and percentage-based max width constraints
    - Smooth drag experience with visual feedback
    - Prevents text selection during drag
    - Width constraints enforced (min/max)
    - Separate handling for left-side and right-side panels
    - Toggle buttons track panel widths via computed properties
- ✅ **Improved Control Layout:**
  - Card size controls integrated into grid footer
  - Positioned at bottom-right of grid container (relative positioning)
  - Displayed alongside file count for better context
  - No longer overlaps with other UI elements
  - Part of the scrollable content area
- ✅ **Compact Top Menu Bar:**
  - Sticky top bar in the grid with all view controls
  - Transparent background with backdrop blur for modern look
  - Left section: Sidebar toggle button and file count
  - Center section: Card size controls (Small/Medium/Large icons)
  - Right section: Info panel toggle button
  - All controls consolidated in one place for easy access
  - Toggle buttons moved from App.vue to ThumbnailGrid component
  - Cleaner main layout without floating buttons
  - Menu bar only visible when media files are present

**Pending (Future Iterations):**
- ⏳ Hover zoom/preview functionality
- ⏳ Sort by date controls (currently sorts newest first by default)
- ⏳ Full-screen slideshow mode
- ⏳ Virtual scrolling for performance with 1000+ files
- ⏳ Actions for selected files (delete, move, export, etc.)
- ⏳ Database caching for extracted metadata (currently extracted on-demand)

**Technical Stack:**
- Rust: `image`, `walkdir`, `serde`, `chrono`
- Vue 3 Composition API with TypeScript
- Tauri plugins: `dialog`, `fs`

---

- Easily See Photo/Video Information (FR-1.1): Planned
- Library Indexing (FR-2.6): Planned

## Phase 2: Organization and Search

- Advanced Search and Filtering (FR-3.1): Planned
- Organization: Albums by Theme/Date/Content (FR-2.1): Planned
- Metadata Editing (FR-2.5): Planned

## Phase 3: Auto-Organization and Deduplication

- Auto-Label Photo/Video Content (FR-2.2): Planned
- Auto-Group Similar Photos and Auto-Rank (FR-2.3): Planned
- Duplicate and Similar Detection for Videos (FR-2.4): Planned

## Phase 4: Editing and AI Transformations

- Basic Non-AI Edits (FR-4.4): Planned
- Upscale/Downscale Photos (FR-4.2): Planned
- Reencode/Transform Videos (FR-4.3): Planned
- AI Transformation via Prompting (FR-4.1): Planned

---

**Legend:**
- Planned: Feature is described in README.md but not yet implemented in the app.
- Implemented: Feature is available in the app.
- In Progress: Feature is currently being developed.

Update this document as features are implemented or their status changes.
