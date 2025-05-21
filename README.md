
# Dolphin Editor (dolphin-edit): The Lightning-Fast Long JSONL Viewer

Dolphin Editor is a cross-platform, minimalist code editor laser-focused on one task: efficiently opening, viewing, and searching **extremely long** JSON Lines (`.jsonl`) files. It prioritizes performance and minimal memory usage by never loading the entire file into memory, instead utilizing intelligent indexing (including **line-offset, inverted, and n-gram indexes**), streaming, and **caching**.

## Core Features

1.  **Extreme Large File Support:** Seamlessly opens and navigates multi-gigabyte (even terabyte-scale, disk permitting) JSONL files.
2.  **Dual-Panel View:**
    *   **Main Panel (Raw View):** Displays the raw, unprocessed lines of the JSONL file with virtualized rendering.
    *   **Right Panel (Pretty JSON View):** Pretty-prints the JSON object from the line where the cursor currently resides in the main panel.
3.  **Exceptional Memory Efficiency:** Employs file streaming and foundational line indexing to keep RAM usage minimal, regardless of file size.
4.  **Blazing Fast Search & Replace:**
    *   Full Regular Expression (regex) support.
    *   Utilizes **inverted indexes** for keyword-driven regex components and **n-gram indexes** for accelerating substring and complex pattern matching within regexes.
    *   Case sensitivity, whole word options.
    *   Efficient "Find All" and "Replace All" operations.
5.  **Cross-Platform Native Feel:** Runs on macOS, Windows, and Linux using Tauri (Rust backend, web-view frontend).
6.  **Hyper-Minimalist & Focused UI/UX:** A deliberately uncluttered interface designed for speed and clarity.
7.  **Advanced Smart Caching:** Caches various index types (line-offset, inverted, n-gram) to dramatically speed up subsequent opens and searches on previously analyzed files.

## The Problem It Solves

Standard text editors and IDEs often crumble under the weight of very large text files. Dolphin Editor is engineered from the ground up with advanced indexing and streaming to bypass these limitations, making it an indispensable tool for data scientists, log analysts, and developers needing to search and analyze massive JSONL datasets efficiently.

## Technology Stack

*   **Core Logic & Backend:** [Rust](https://www.rust-lang.org/)
    *   High-performance, memory-safe processing.
    *   Concurrency for non-blocking operations (e.g., indexing, search).
    *   Key Crates: `tokio` (async runtime), `std::fs`, `std::io::BufReader`, `regex`, `serde_json`, `bincode` (for index serialization), `directories` (for cache dir), potentially specialized n-gram or suffix array crates if beneficial.
*   **Frontend & UI:** [Tauri](https://tauri.app/)
    *   Uses system's web-view for rendering HTML, CSS, and JavaScript.
    *   Lightweight compared to Electron.
    *   **UI Framework (Frontend):** Plain HTML/CSS/JavaScript. For true minimalism, vanilla JS is sufficient.
*   **Communication:** Tauri's IPC bridge between Rust backend and JS frontend.

## UI/UX Design

The UI/UX philosophy is **"Focus and Flow."** Every element serves a purpose, and interactions are designed to be intuitive and fast.

### 1. Main Window Layout

*   **Title Bar:** Standard OS title bar displaying "Dolphin Editor - [filename]".
*   **Menu Bar (Minimal):**
    *   `File`
        *   `Open...` (Ctrl/Cmd+O)
        *   `Close File` (Ctrl/Cmd+W)
        *   `Save (after Replace All)` (Ctrl/Cmd+S) - Enabled only after a "Replace All" operation generates a modified version.
        *   `Save As... (after Replace All)`
        *   `Exit` (Alt+F4 / Cmd+Q)
    *   `Edit`
        *   `Find...` (Ctrl/Cmd+F)
        *   `Replace...` (Ctrl/Cmd+H)
    *   `View`
        *   `Toggle Pretty JSON Panel` (Ctrl/Cmd+P)
*   **Main Content Area (Two Panels):**
    *   **Left: Raw View Panel (Resizable Width)**
        *   **Line Numbers:** Fixed-width column on the far left, always visible.
        *   **Content Area:** Displays raw lines. Vertical scrollbar handles virtualized content.
        *   **Cursor:** A clear visual indicator (e.g., a blinking bar or full-line highlight) on the currently active line.
    *   **Right: Pretty JSON Panel (Resizable Width, Toggleable)**
        *   **Content Area:** Displays `JSON.stringify(parsed, null, 2)` of the active line's JSON.
        *   **Error Display:** If the current line is not valid JSON, displays a clear, user-friendly message (e.g., "Invalid JSON on this line: [error message snippet]").
        *   Vertical scrollbar if pretty-printed JSON exceeds panel height.
*   **Search/Replace Bar (Appears at top/bottom of Raw View Panel when Ctrl/Cmd+F or H is pressed):**
    *   `[Find Input Field (regex enabled)]` `[Replace Input Field (if Replace active)]`
    *   Buttons/Toggles:
        *   `Aa` (Case Sensitive)
        *   `\b` (Whole Word - if implemented, less critical for JSONL)
        *   `.*` (Use Regular Expression - often default on)
        *   `Previous Match` (Arrow Up icon)
        *   `Next Match` (Arrow Down icon)
        *   `Replace` (Only if Replace active, replaces current find)
        *   `Replace All` (Only if Replace active)
    *   Match Count: `1 of 15 matches`
    *   Close Button (`X`)
*   **Status Bar (Bottom of Window):**
    *   Left: Full file path of the open file.
    *   Center: `Ln X, Col Y` (cursor position in raw view), `Total Lines: ZZZ`.
    *   Right: Indexing progress (e.g., "Indexing (N-grams)... 45%") or "Ready". Search progress if a long search is running.

### 2. User Experience (UX) Highlights

*   **Instant Open (Goal):** After initial comprehensive indexing (which is visualized, or skipped if cached), navigation should feel instant.
*   **Smooth Scrolling:** Virtualized scrolling ensures no lag, regardless of file size. The scrollbar thumb size should reflect the position within the *entire* file, not just the loaded portion.
*   **Responsive Pretty Print:** The pretty print panel updates immediately as the cursor moves to a new line in the raw view.
*   **Non-Blocking Operations:** File indexing (all types), searching, and "Replace All" run asynchronously, with progress updates to the UI. The UI remains responsive.
*   **Clear Feedback:** Visual cues for loading, type of indexing being performed, search results, and errors.
*   **Keyboard First:** All core actions are accessible via keyboard shortcuts, similar to popular editors.
*   **Minimal Distractions:** No unnecessary toolbars, icons, or features.
*   **Instant Search (Goal):** Leveraging advanced indexes, search results should appear almost instantaneously, even in massive files, after initial indexing is complete.

## Architecture (Tauri + Rust)

*   **Rust Backend (Core Logic):**
    *   `main.rs`: Sets up Tauri application, defines Tauri commands.
    *   `file_handler.rs`: (Or could be split further)
        *   `IndexingService`: Orchestrates the building of all index types (line-offset, inverted, n-gram). Manages caching of these indexes.
        *   `FileContentService`: Reads specific line data (still uses line-offset index for this).
    *   `search_handler.rs`:
        *   `SearchPlanner`: Analyzes the search query/regex to determine the optimal strategy (e.g., use inverted index, n-gram index, or combination, falling back to full scan if necessary).
        *   `SearchExecutor`: Executes the search plan.
    *   `cache_manager.rs`: (Optional dedicated module) Manages cache file operations, key generation, and validation for all index types.
    *   `state.rs`: Manages shared application state (e.g., current file path, loaded indexes, search results) using `std::sync::Mutex` or `tokio::sync::Mutex` for thread-safe access.
    *   `utils/`: Contains helper modules for tokenization, n-gram generation, etc.
*   **Frontend (Web View - `src/` directory):**
    *   `index.html`, `styles.css`, `main.js` (or Svelte/SolidJS components).
    *   `RawView.js`: Manages the virtualized list for raw lines.
    *   `PrettyJsonView.js`: Manages the pretty-printed JSON display.
    *   `SearchBar.js`: Handles search UI interactions.
    *   Communicates with Rust backend via `window.__TAURI__.invoke('rust_command_name', { args })`.
*   **Tauri Commands (Rust functions exposed to JS):**
    *   `open_file(path: String)`: Triggers line indexing or loads from cache.
    *   `get_lines(start_line: usize, count: usize)`: Fetches a chunk of lines.
    *   `get_line_content(line_number: usize)`: Fetches content for a single line (for pretty print).
    *   `search_file(query: String, is_regex: bool, case_sensitive: bool)`: Initiates a search using the advanced indexing.
    *   `replace_all_in_file(find_query: String, replace_with: String, is_regex: bool, case_sensitive: bool)`: Initiates replace all.
    *   `get_total_lines()`: Returns total line count after indexing.
    *   `get_indexing_status()`: To provide more granular feedback on which indexes are being built/loaded and their progress.
    *   `clear_all_caches()`: (Optional) Command to manually clear all index caches.

## Performance & Algorithms for Lightning Speed

Dolphin Editor employs a multi-layered indexing strategy for unparalleled performance.

### 1. Foundational: Line-Offset Index

*   **Algorithm & Purpose:** Maps line number to byte offset and length.
    *   **Build:** Always built first or loaded from cache. Essential for virtual scrolling and fetching line content for other indexing stages and display.
*   **Data Structure:** `Arc<Mutex<Vec<(u64, usize)>>>`.
*   **Benefit:** O(1) access to any line's raw data location.

### 2. Accelerating Keyword Search: Inverted Index

*   **Algorithm & Purpose:** Maps "terms" (meaningful tokens extracted from JSON content) to a list of line numbers where they appear.
    *   **Tokenization:** During indexing, each line's JSON is parsed. Keys and string values (and potentially numbers if configured) are extracted as terms. Normalization (e.g., lowercasing) can be applied.
    *   **Build:** After the line-offset index is ready, this can be built by iterating through each line, parsing its JSON, extracting terms, and populating the index.
    *   **Data Structure:** `Arc<Mutex<HashMap<String, Vec<u32>>>>` (Term -> sorted list of line numbers). `u32` for line numbers can save space if total lines < 4 billion.
*   **Search Usage:**
    *   If a regex contains literal keywords (e.g., `user_id="abc"` AND `event_type="login"`), the `SearchPlanner` identifies these.
    *   Retrieves line lists for "abc" (if indexed as a value) and "login".
    *   Calculates the intersection of these line lists.
    *   The full regex is then applied *only* to this much smaller candidate set of lines.
*   **Benefit:** Massively reduces the number of lines that need full regex evaluation for queries containing common or selective keywords.

### 3. Powering Substring & Complex Regex: N-gram Index

*   **Algorithm & Purpose:** Maps short sequences of characters (n-grams, typically trigrams: 3 characters) to a list of line numbers containing them.
    *   **Build:** After the line-offset index, iterate through each line's raw content. For each line, generate all overlapping n-grams (e.g., "hello" -> "hel", "ell", "llo").
    *   **Data Structure:** `Arc<Mutex<HashMap<[u8; N], Vec<u32>>>>` (N-gram bytes -> sorted list of line numbers). Using byte arrays for n-grams can be more efficient than strings. `N` would be the n-gram size (e.g., 3 for trigrams).
*   **Search Usage (`SearchPlanner`):**
    *   For a search string like "critical_error":
        1.  Break it into n-grams: "cri", "rit", "iti", "tic", ... , "ror".
        2.  Retrieve line lists for each of these n-grams from the index.
        3.  The intersection of these lists gives a candidate set of lines highly likely to contain "critical_error".
    *   For regexes with literal substrings: The `SearchPlanner` can extract these substrings, use their n-grams to filter, and then apply the full regex.
    *   This can also help with some wildcard patterns if the surrounding literals are long enough to generate useful n-grams.
*   **Benefit:** Dramatically speeds up arbitrary substring searches and can significantly prune the search space for many regex patterns, avoiding full line scans.
*   **Trade-off:** N-gram indexes can be the largest and take the longest to build. The choice of `N` (e.g., 2, 3, 4) affects size and selectivity. Trigrams are a common balance.

### 4. Virtualized Scrolling (`RawView.js` Frontend + Rust Backend)
*(This section remains largely the same as before, relying on Line-Offset Index)*
*   **Algorithm (Frontend):**
    1.  Calculate how many lines fit in the visible viewport (`visible_lines`).
    2.  Add a buffer (e.g., `buffer_lines = visible_lines / 2`).
    3.  On scroll event, determine the `start_line_index` for the current scroll position.
    4.  Request `visible_lines + 2 * buffer_lines` from the Rust backend, starting at `max(0, start_line_index - buffer_lines)`.
    5.  The frontend only renders DOM elements for these lines. Other lines are represented by spacer DIVs to make the scrollbar behave correctly, or by adjusting the scroll container's total height.
*   **Data Fetching (Rust Backend - `get_lines` command):**
    1.  Receive `start_line`, `count`.
    2.  Access the Line-Offset Index.
    3.  For each requested line number `i` from `start_line` to `start_line + count - 1`:
        *   Get `(offset, length) = line_offset_index[i]`.
        *   Open the file (or use an existing handle).
        *   Use `file.seek(SeekFrom::Start(offset))`.
        *   Read `length` bytes into a buffer using `file.read_exact()`.
        *   Convert buffer to `String`.
    4.  Return `Vec<String>` of lines to the frontend.

### 5. Efficient Search Execution (`SearchPlanner` & `SearchExecutor`)

1.  **Query Analysis (`SearchPlanner`):**
    *   Parses the regex.
    *   Identifies literal keywords for potential Inverted Index lookup.
    *   Identifies literal substrings for potential N-gram Index lookup.
    *   Estimates the cost/selectivity of using each index.
    *   Decides on a plan:
        *   Use Inverted Index, then N-gram on results, then full regex.
        *   Use N-gram Index, then full regex.
        *   Directly use full regex (fallback if indexes offer no clear benefit or for very complex patterns).
2.  **Candidate Line Retrieval:** Fetch candidate line numbers from the chosen index(es) and combine them (e.g., intersection).
3.  **Full Regex Application:** Fetch the actual content of candidate lines (using Line-Offset Index) and apply the full regex to confirm matches.
4.  **Asynchronous & Streaming:** All indexing and searching operations run asynchronously. Search results can be streamed to the UI as they are found.

### 6. Optimized "Replace All" (`SearchService` - Rust Backend)

*   **Algorithm:**
    1.  The **search** part to identify lines to be replaced will leverage the `SearchPlanner` and advanced indexes for speed.
    2.  Compile the search regex *once*.
    3.  Open the original file for reading (`BufReader`).
    4.  Create a **new temporary file** for writing (`BufWriter`).
    5.  Iterate through the original file line by line using `buf_reader.lines()` (or more efficiently by using the line-offset index to jump and read):
        *   If the current line is identified as a match by the initial search phase:
            *   Perform `regex.replace_all(&line_content, &replace_string)`.
            *   Write the modified line to the temporary `BufWriter`.
        *   Else:
            *   Write the original line to the temporary `BufWriter`.
    6.  Flush and close the `BufWriter`. Close the reader.
    7.  Once complete, prompt the user (via frontend) to overwrite or save as new.
*   **Benefit:** True streaming for writing. The search part is accelerated by indexes.

## Caching Strategies for Performance

Dolphin Editor employs comprehensive caching for all generated indexes:

1.  **Persistent Line-Offset Index Caching:**
    *   **Mechanism:** After a file's line-offset index is built, it's serialized (e.g., `bincode`) and saved to a local cache directory.
    *   **Trigger:** On subsequent opens, if file metadata (path, mod time, size) is unchanged, this index is loaded directly.
    *   **Benefit:** Essential for fast subsequent opens and as a prerequisite for loading other cached indexes.

2.  **Persistent Inverted Index Caching:**
    *   **Mechanism:** Similar to line-offset index. Serialized and stored. Cache key includes file metadata and perhaps tokenization settings.
    *   **Benefit:** Avoids re-tokenizing and re-building the inverted index on subsequent opens, speeding up keyword-based searches.

3.  **Persistent N-gram Index Caching:**
    *   **Mechanism:** Serialized and stored. Cache key includes file metadata and n-gram size.
    *   **Benefit:** Avoids re-generating n-grams for the entire file. This is often the biggest time-saver for subsequent substring/complex regex searches.

4.  **Cache Management:**
    *   **Validation:** On file open, check metadata (path, mod time, size) against cached index metadata. If changed, relevant indexes are invalidated and rebuilt.
    *   **Granularity:** Cache different index types separately, so if only one is stale, only that one needs rebuilding. (e.g., line-offset index might be valid, but n-gram needs refresh if n-gram settings change).
    *   **User Control (Future):** Options to configure cache location, max size, and manually clear caches.
    *   **Dependencies:** Loading a cached n-gram or inverted index also requires a valid (cached or fresh) line-offset index.

## Core Components (Conceptual Rust/Tauri Structure)

```
dolphin-edit/
├── src/                      # Frontend (HTML, CSS, JS)
│   ├── index.html
│   ├── main.js               # Main JS entry point for UI logic
│   ├── styles.css
│   └── components/           # (Optional) if using a JS framework
│       ├── RawView.js
│       └── PrettyJsonView.js
├── src-tauri/                # Rust backend
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs           # Rust entry point, Tauri setup
│       ├── indexing_service.rs # Builds and manages all index types
│       ├── file_content_service.rs # Reads raw line data
│       ├── search_handler.rs   # Contains SearchPlanner, SearchExecutor
│       ├── cache_manager.rs  # Manages cache logic for all indexes
│       ├── state.rs          # Shared application state
│       ├── commands.rs       # Tauri commands exposed to JS
│       └── utils/              # For tokenizers, n-gram generators etc.
└── package.json              # For managing frontend JS dependencies (if any) & tauri CLI scripts
```

## Key Design Decisions & Trade-offs

*   **Multi-layered Indexing:** Provides flexibility and unparalleled search speed but significantly increases initial indexing time for new files and cache disk usage. The `SearchPlanner` is crucial for leveraging them effectively.
*   **Rust + Tauri:** Chosen for performance, memory safety, and lighter binaries than Electron. Steeper learning curve than JS-only, but powerful for this task.
*   **Configurability (Future):** Users might eventually be able to choose which advanced indexes to build (e.g., "fast open, slower search" vs. "slower open, fastest search") to balance indexing time with search speed needs for specific workflows.
*   **"Replace All" Creates New File:** Enhances data safety and simplifies implementation for large files.
*   **Web View for UI:** Faster UI development for this type of layout than pure native Rust GUI libraries. Relies on system web-view.

## Future Enhancements (Post-MVP)

*   Syntax highlighting for the current line in the `PrettyJsonView` (JS side).
*   Basic syntax highlighting in `RawView` for JSON tokens (more complex, needs careful performance consideration).
*   "Go to Line" functionality.
*   User configuration for indexing strategies (e.g., enable/disable n-gram or inverted indexes, choose n-gram size, select JSON fields for inverted index).
*   Advanced cache management UI (view cache sizes, clear specific caches, set limits).
*   Incremental indexing for files that are appended to (updating indexes without full rebuild).
*   Support for indexing specific JSON fields only for the inverted index (schema-aware indexing).
*   Tabbed interface for multiple files.

## Getting Started (Development with Rust & Tauri)

1.  **Install Rust:** Follow instructions at [rustup.rs](https://rustup.rs/).
2.  **Install Tauri Prerequisites:** Follow the Tauri [setup guide](https://tauri.app/v1/guides/getting-started/prerequisites) for your OS (Node.js for frontend tooling, WebView2 for Windows, etc.).
3.  **Clone the repository:** `git clone <repository_url_for_dolphin-edit>`
4.  **Navigate to project directory:** `cd dolphin-edit`
5.  **(If frontend has JS dependencies):** `npm install` (or `yarn install`) in the root.
6.  **Run development server:** `cargo tauri dev`
