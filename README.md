Implementation Plan for FMLM Features
-------------------------------------

This plan sequences features by utility (prioritizing daily-use essentials like browsing and organization for quick value) and complexity (starting with low-effort basics like UI views, progressing to AI integrations requiring external libs/models). Phases build an MVP first, then enhance iteratively. Each feature includes: description (tied to SRS), key details/sub-tasks, dependencies, and estimated relative complexity/effort (Low/Med/High, assuming AI-assisted Rust/Tauri dev).

### Phase 1: Core Viewing and Browsing (MVP Foundation - High Utility, Low Complexity)

Focus: Enable basic library access and navigation. Users can immediately import/view media without advanced logic. Effort: Build a functional app shell in 1-2 weeks.

#### Thumbnail Grid View (FR-3.2)

*   **Description**: Customizable grid or timeline for browsing thumbnails with hover zoom/preview; sort by date; full-screen slideshow.
    
*   **Details/Sub-Tasks**:
    
    *   Generate thumbnails from files using image crate (e.g., resize to 200x200px).
        
    *   UI: Vue.js component with lazy loading (e.g., virtual scrolling for 100+ items); hover shows enlarged preview.
        
    *   Add sorting/filter toggles (date ascending/descending).
        
    *   Slideshow: Keyboard controls (arrow keys) and timer-based playback.
        
*   **Dependencies**: File scanning basics (from next feature); frontend framework (Vue.js).
    
*   **Complexity/Effort**: Low (UI-focused; ~2-3 days).
    

#### Easily See Photo/Video Information (FR-1.1)

*   **Description**: Display metadata like resolution, file size, codecs, duration (videos), date, camera model on selection.
    
*   **Details/Sub-Tasks**:
    
    *   Rust command: Extract EXIF/IPTC via exif crate (e.g., get\_resolution(path) -> u32x u32).
        
    *   UI: Side panel or modal showing formatted info (e.g., table with icons); support batch for selected items.
        
    *   Handle formats: JPEG/HEIC (photos), MP4/MOV (videos via infer crate for codecs).
        
    *   Error handling: Flag unsupported files.
        
*   **Dependencies**: Thumbnail grid for selection triggering.
    
*   **Complexity/Effort**: Low (Library integration; ~1-2 days).
    

#### Library Indexing (FR-2.6)

*   **Description**: Background indexing of libraries for fast access; scan for new files on import; progress indicators.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Use walkdir crate to scan directories (parallel for speed); store in SQLite (rusqlite) with paths/metadata.
        
    *   Commands: index\_library(path, recursive: bool) -> ProgressUpdate; incremental scans via file watchers (notify crate).
        
    *   UI: Progress bar in status panel; initial scan <5 min for 10k files.
        
    *   Output: Indexed list fed to grid/search.
        
*   **Dependencies**: None (core enabler).
    
*   **Complexity/Effort**: Low-Med (File I/O basics; ~3-4 days).
    

### Phase 2: Organization and Search (Core Usability - High Utility, Medium Complexity)

Focus: Add structure and discovery. Builds on Phase 1 for a usable organizer. Effort: Add in 1-2 weeks post-MVP.

#### Advanced Search and Filtering (FR-3.1)

*   **Description**: Search by metadata/labels/content (e.g., "beach 2024"); filters for size/type/date.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Query SQLite index (e.g., SQL with LIKE/Fuzzy via fts5 extension); semantic search via embeddings (later phase).
        
    *   UI: Search bar with autocomplete; filter dropdowns (e.g., file type, size range); results update grid in real-time.
        
    *   Support: Date ranges, exact metadata matches (e.g., resolution >1080p).
        
*   **Dependencies**: Library indexing; metadata viewing.
    
*   **Complexity/Effort**: Med (Query logic; ~3 days).
    

#### Organization: Albums by Theme/Date/Content (FR-2.1)

*   **Description**: Create/manage albums/folders; drag-drop assignment; auto-suggest via metadata.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Store albums in SQLite (relations: media -> album); create\_album(name) -> Id, add\_to\_album(files: Vec, album\_id).
        
    *   UI: Sidebar tree view for albums; drag from grid; theme/date auto-folders (e.g., yearly buckets).
        
    *   Non-destructive: Track links, not move files.
        
*   **Dependencies**: Indexing and search for population.
    
*   **Complexity/Effort**: Med (DB relations; ~4 days).
    

#### Metadata Editing (FR-2.5)

*   **Description**: Edit EXIF/IPTC fields (e.g., captions, dates, remove GPS); batch support.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Use exif crate for read/write (e.g., set\_date(path, new\_date)); batch via iterator.
        
    *   UI: Editable fields in info panel; confirmation dialog; undo via temp backups.
        
    *   Formats: Update in-place; validate changes (e.g., date format).
        
*   **Dependencies**: Metadata viewing.
    
*   **Complexity/Effort**: Med (I/O mutations; ~2-3 days).
    

### Phase 3: Auto-Organization and Deduplication (Efficiency Boost - Medium-High Utility, Medium Complexity)

Focus: Automate tedious tasks for cleaning/growing libraries. Effort: 2 weeks, after basic org.

#### Auto-Label Photo/Video Content (FR-2.2)

*   **Description**: Generate labels/tags (e.g., "family beach") via local AI; editable.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Call Ollama API (reqwest) with CLIP/LLaVA model (e.g., base64 image -> prompt "Describe scene").
        
    *   Batch: Process queue with progress; store tags in SQLite.
        
    *   UI: Tag badges in grid/info; manual edit/add.
        
    *   Accuracy: Threshold filter (e.g., confidence >0.8); fallback to metadata.
        
*   **Dependencies**: Indexing; Ollama setup (external model pull).
    
*   **Complexity/Effort**: Med-High (AI integration; ~5 days).
    

#### Auto-Group Similar Photos and Auto-Rank (FR-2.3)

*   **Description**: Group clusters (e.g., bursts); rank by quality; select best/delete others.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Perceptual hash (image crate) or embeddings (CLIP via Ollama); group with similarity threshold (e.g., >90%).
        
    *   Ranking: Score sharpness/blur via OpenCV-rs; UI previews ranked list.
        
    *   Actions: Confirm dialog for multi-delete; move to trash/recycle.
        
*   **Dependencies**: Auto-labeling for content aid.
    
*   **Complexity/Effort**: Med-High (Similarity algos; ~4-5 days).
    

#### Duplicate and Similar Detection for Videos (FR-2.4)

*   **Description**: Flag exact dups (hash) or similars (keyframes); batch delete/merge.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: FFmpeg-next for keyframe extraction (e.g., every 5s); hash thumbnails.
        
    *   Integrate with photo grouping: Unified scan command.
        
    *   UI: Grouped view with video previews (seekable); similar to photo ranking.
        
*   **Dependencies**: Auto-group photos; FFmpeg crate.
    
*   **Complexity/Effort**: Med-High (Video processing; ~4 days).
    

### Phase 4: Editing and AI Transformations (Enhancement Layer - Medium Utility, High Complexity)

Focus: Media manipulation for maintenance/sharing. Highest complexity due to processing/AI. Effort: 2-3 weeks last.

#### Basic Non-AI Edits (FR-4.4)

*   **Description**: Crop, rotate, brightness/contrast; batch apply; trim for videos.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Image crate for photos (e.g., crop(rect)); FFmpeg for video trims.
        
    *   UI: Toolbar tools (sliders for adjust); preview before apply; non-destructive via copies.
        
    *   Batch: Queue selections.
        
*   **Dependencies**: Metadata viewing.
    
*   **Complexity/Effort**: High (UI + processing; ~5 days).
    

#### Upscale/Downscale Photos (FR-4.2)

*   **Description**: Resize with interpolation; AI upscale option; batch.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Image crate (Lanczos); ESRGAN via Ollama for AI upscale.
        
    *   Params: Target res (e.g., 4K); quality presets.
        
    *   UI: Dropdown for method; progress for batch.
        
*   **Dependencies**: Basic edits; Ollama.
    
*   **Complexity/Effort**: High (AI optional; ~4 days).
    

#### Reencode/Transform Videos (FR-4.3)

*   **Description**: Convert formats/codecs (e.g., compress MP4); presets for compatibility.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: FFmpeg-next (e.g., encode(input, output, preset: "fast")); async with progress callbacks.
        
    *   UI: Format selector; quality slider; queue management.
        
    *   Handle large files: Temp storage, cancel option.
        
*   **Dependencies**: Basic edits.
    
*   **Complexity/Effort**: High (Video I/O; ~5 days).
    

#### AI Transformation via Prompting (FR-4.1)

*   **Description**: Prompt-based edits (e.g., "add filter"); new file alongside original.
    
*   **Details/Sub-Tasks**:
    
    *   Rust: Ollama with Stable Diffusion (e.g., POST prompt + image).
        
    *   UI: Text input/prompt history; preview generated output.
        
    *   Scope: Images first, extend to video frames; batch limited.
        
*   **Dependencies**: All prior AI (labeling/grouping); Ollama models.
    
*   **Complexity/Effort**: High (Prompt handling; ~6 days).