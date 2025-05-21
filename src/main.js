// src/main.js
console.log("Dolphin Editor Main JS Loaded");

const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window; // For listening to resize events

// DOM Elements
const rawViewPanel = document.getElementById('raw-view-panel');
const rawViewContent = document.getElementById('raw-view-content');
const rawViewLineNumbers = rawViewPanel.querySelector('.line-numbers'); // Corrected selector
const prettyJsonContentPre = document.querySelector('#pretty-json-content pre');

const statusFilePath = document.getElementById('status-file-path');
const statusTotalLines = document.getElementById('status-total-lines');
const statusIndexing = document.getElementById('status-indexing');


// --- Global State (Simplified) ---
let totalLines = 0;
let indexingStatusInterval = null; // For polling indexing status
let activeLineIndex = -1;
let activeLineElement = null; // To style the active line
let linesCache = {}; // Cache for lines already fetched
let lineHeight = 0; // To be calculated
let visibleLinesCount = 0; // Lines visible in viewport
let currentlyDisplayedLines = []; // Array of DOM nodes
let currentScrollTop = 0;
const OVERSCAN_COUNT = 10; // Number of lines to render above/below viewport

// --- Utility Functions ---
function calculateLineHeight() {
    // Create a temporary line, measure it, then remove it.
    const tempLine = document.createElement('div');
    tempLine.textContent = 'X'; // Content doesn't matter, just need height
    tempLine.style.visibility = 'hidden'; // Don't show it
    rawViewContent.appendChild(tempLine);
    lineHeight = tempLine.offsetHeight;
    rawViewContent.removeChild(tempLine);
    if (lineHeight === 0) { // Fallback if calculation fails
        console.warn("Line height calculation failed, using fallback.");
        lineHeight = 15; // Adjust as needed
    }
    console.log("Calculated line height:", lineHeight);
}

function updateVisibleLinesCount() {
    if (lineHeight > 0) {
        visibleLinesCount = Math.ceil(rawViewContent.clientHeight / lineHeight);
    }
    console.log("Visible lines count:", visibleLinesCount);
}

// --- Function to Update Indexing Status Display ---
async function updateIndexingStatusDisplay() {
    try {
        const status = await invoke('get_indexing_status');
        if (status && typeof status.message === 'string' && typeof status.progress === 'number') {
            const progressPercent = Math.round(status.progress * 100);
            statusIndexing.textContent = `${status.message} (${progressPercent}%)`;

            // If indexing is complete (progress is 1.0 and message is "Ready" or similar)
            // or if there was an error message that implies completion.
            if (status.progress >= 1.0 || status.message.toLowerCase().includes("error") || status.message.toLowerCase() === "ready") {
                if (indexingStatusInterval) {
                    clearInterval(indexingStatusInterval);
                    indexingStatusInterval = null;
                    // Ensure final status is "Ready" if progress is 1.0 and no error in message
                    if (status.progress >= 1.0 && !status.message.toLowerCase().includes("error")) {
                        statusIndexing.textContent = "Ready";
                    }
                }
            }
        }
    } catch (error) {
        console.error("Error fetching indexing status:", error);
        statusIndexing.textContent = "Error fetching status.";
        if (indexingStatusInterval) {
            clearInterval(indexingStatusInterval);
            indexingStatusInterval = null;
        }
    }
}

// --- Function to Update Pretty JSON View ---
async function updatePrettyJsonView(lineNumber) {
    if (lineNumber < 0 || lineNumber >= totalLines) {
        prettyJsonContentPre.textContent = ''; // Clear if invalid line number
        return;
    }

    try {
        statusIndexing.textContent = `Fetching line ${lineNumber + 1} for pretty view...`;
        const lineContent = await invoke('get_line_content', { lineNumber });
        statusIndexing.textContent = 'Parsing JSON...';
        try {
            const parsedJson = JSON.parse(lineContent);
            prettyJsonContentPre.textContent = JSON.stringify(parsedJson, null, 2);
        } catch (e) {
            prettyJsonContentPre.textContent = `Invalid JSON on this line: ${e.message}

Raw content:
${lineContent}`;
        }
    } catch (error) {
        console.error(`Error fetching line ${lineNumber} for pretty view:`, error);
        prettyJsonContentPre.textContent = `Error fetching line content: ${error}`;
    } finally {
        if (statusIndexing.textContent.startsWith('Fetching line') || statusIndexing.textContent.startsWith('Parsing JSON')) {
             statusIndexing.textContent = 'Ready';
        }
    }
}

// --- Core Virtual Scrolling Logic ---
async function renderVisibleLines() {
    if (totalLines === 0 || lineHeight === 0) {
        rawViewContent.innerHTML = ''; // Clear content if no lines or line height unknown
        rawViewLineNumbers.innerHTML = '';
        return;
    }

    const firstVisibleLineIndex = Math.floor(currentScrollTop / lineHeight);
    const startIndex = Math.max(0, firstVisibleLineIndex - OVERSCAN_COUNT);
    const endIndex = Math.min(totalLines, firstVisibleLineIndex + visibleLinesCount + OVERSCAN_COUNT);

    // Clear existing lines (simple approach, can be optimized)
    rawViewContent.innerHTML = '';
    rawViewLineNumbers.innerHTML = '';

    // Create a document fragment for batch DOM append
    const contentFragment = document.createDocumentFragment();
    const lineNumbersFragment = document.createDocumentFragment();

    let linesToFetch = [];
    for (let i = startIndex; i < endIndex; i++) {
        if (!linesCache[i]) {
            linesToFetch.push(i);
        }
    }
    
    // Group contiguous lines to fetch
    let fetchRanges = [];
    if(linesToFetch.length > 0) {
        let currentRange = {start: linesToFetch[0], count: 1};
        for(let i = 1; i < linesToFetch.length; i++) {
            if(linesToFetch[i] === linesToFetch[i-1] + 1) {
                currentRange.count++;
            } else {
                fetchRanges.push(currentRange);
                currentRange = {start: linesToFetch[i], count: 1};
            }
        }
        fetchRanges.push(currentRange);
    }


    if (fetchRanges.length > 0) {
        statusIndexing.textContent = 'Fetching lines...';
        try {
             for (const range of fetchRanges) {
                 const fetchedLines = await invoke('get_lines', { startLine: range.start, count: range.count });
                 fetchedLines.forEach((lineContent, idx) => {
                     linesCache[range.start + idx] = lineContent;
                 });
             }
        } catch (error) {
            console.error("Error fetching lines:", error);
            statusIndexing.textContent = `Error: ${error}`;
            return; // Stop rendering if fetch fails
        } finally {
            if (statusIndexing.textContent === 'Fetching lines...') {
                 statusIndexing.textContent = 'Ready';
            }
        }
    }

    for (let i = startIndex; i < endIndex; i++) {
        const lineDiv = document.createElement('div');
        lineDiv.textContent = linesCache[i] || ''; // Use cached or fetched line
        lineDiv.style.height = `${lineHeight}px`;
        lineDiv.style.position = 'absolute';
        lineDiv.style.top = `${i * lineHeight}px`;
        lineDiv.style.width = '100%'; // Ensure it takes full width for selection, etc.

        // Add click listener to update pretty view and active line
        lineDiv.addEventListener('click', () => {
            if (activeLineElement) {
                activeLineElement.classList.remove('active-line');
            }
            activeLineIndex = i; // 'i' is the current line number in the loop
            activeLineElement = lineDiv;
            activeLineElement.classList.add('active-line');
            
            // Update cursor position in status bar
            const statusCursorPos = document.getElementById('status-cursor-pos');
            if (statusCursorPos) {
                // Column is not tracked yet, so set to 1
                statusCursorPos.textContent = `Ln ${activeLineIndex + 1}, Col 1`;
            }

            updatePrettyJsonView(activeLineIndex);
        });

        // Re-apply active style if the active line is re-rendered
        if (i === activeLineIndex) {
            lineDiv.classList.add('active-line');
            activeLineElement = lineDiv; // Keep reference to the new DOM element
        }
        contentFragment.appendChild(lineDiv);

        const lineNumberDiv = document.createElement('div');
        lineNumberDiv.textContent = i + 1; // Line numbers are 1-based
        lineNumberDiv.style.height = `${lineHeight}px`;
        lineNumberDiv.style.position = 'absolute';
        lineNumberDiv.style.top = `${i * lineHeight}px`;
        lineNumberDiv.style.width = '100%';
        lineNumbersFragment.appendChild(lineNumberDiv);
    }

    rawViewContent.appendChild(contentFragment);
    rawViewLineNumbers.appendChild(lineNumbersFragment);

    // Set the total height of the scrollable area to represent all lines
    // This makes the scrollbar behave correctly.
    // The content itself is positioned absolutely.
    // We need a "scroller" div inside rawViewContent that has the full height.
    // For now, let's adjust rawViewContent's direct child if it's the one scrolling, or add one.
    // Let's assume rawViewContent is the scroll container.
    // We need a single child that sets the total scroll height.
    // This part is tricky. A common way is to have a very tall div inside.
    // For now, let's ensure rawViewContent itself is the scroll container.
    // The lines are added directly to rawViewContent.
    // The `rawViewContent` needs a total height.
    
    // Simpler approach for now: the rawViewContent will have its own scrollbar.
    // We'll manage its internal height with a placeholder div.
    // Let's adjust existing placeholder logic if needed.

    // Ensure rawViewContent has position relative to anchor absolute children
    rawViewContent.style.position = 'relative'; 
    rawViewLineNumbers.style.position = 'relative';

    // The scroll height is managed by the content being offset by `top`
    // The container `rawViewContent` needs a total height for the scrollbar.
    // This is usually done by having a single child element that defines the full height.
    // Let's create or update a "sizer" div.
    let sizer = document.getElementById('raw-view-sizer');
    if (!sizer) {
        sizer = document.createElement('div');
        sizer.id = 'raw-view-sizer';
        sizer.style.position = 'absolute';
        sizer.style.top = '0';
        sizer.style.left = '0';
        sizer.style.width = '1px'; // Doesn't need to be wide
        sizer.style.opacity = '0'; // Invisible
        rawViewContent.prepend(sizer); // Add it at the beginning
    }
    sizer.style.height = `${totalLines * lineHeight}px`;

    let lineNumbersSizer = document.getElementById('line-numbers-sizer');
     if (!lineNumbersSizer) {
         lineNumbersSizer = document.createElement('div');
         lineNumbersSizer.id = 'line-numbers-sizer';
         lineNumbersSizer.style.position = 'absolute';
         lineNumbersSizer.style.top = '0';
         lineNumbersSizer.style.left = '0';
         lineNumbersSizer.style.width = '1px';
         lineNumbersSizer.style.opacity = '0';
         rawViewLineNumbers.prepend(lineNumbersSizer);
     }
     lineNumbersSizer.style.height = `${totalLines * lineHeight}px`;
}


// --- Event Handlers ---
rawViewContent.addEventListener('scroll', () => {
    currentScrollTop = rawViewContent.scrollTop;
    // Synchronize line numbers scroll
    rawViewLineNumbers.scrollTop = currentScrollTop; 
    requestAnimationFrame(renderVisibleLines); // Use rAF for smoother updates
});

// Synchronize scrolling from line numbers to content (e.g. mouse wheel on line numbers)
 rawViewLineNumbers.addEventListener('scroll', () => {
     currentScrollTop = rawViewLineNumbers.scrollTop;
     rawViewContent.scrollTop = currentScrollTop;
     // requestAnimationFrame(renderVisibleLines); // Already handled by rawViewContent's scroll
 });


async function handleFileOpen(filePath) {
    if (!filePath) { // User cancelled dialog
        return;
    }
    statusFilePath.textContent = filePath;
    // statusIndexing.textContent = 'Indexing...'; // Replaced by polling logic
    linesCache = {}; // Clear cache for new file

    // Clear Pretty JSON View and active line state
    activeLineIndex = -1;
    if (activeLineElement) {
        activeLineElement.classList.remove('active-line');
        activeLineElement = null;
    }
    prettyJsonContentPre.textContent = ''; // Clear pretty view
    const statusCursorPos = document.getElementById('status-cursor-pos');
    if (statusCursorPos) {
       statusCursorPos.textContent = `Ln 0, Col 0`;
    }

    // Start polling for indexing status
    if (indexingStatusInterval) {
        clearInterval(indexingStatusInterval); // Clear any existing interval
    }
    statusIndexing.textContent = 'Opening file... (0%)'; // Initial message
    indexingStatusInterval = setInterval(updateIndexingStatusDisplay, 500); // Poll every 500ms


    try {
        totalLines = await invoke('open_file', { filePath }); // This is the main call
        statusTotalLines.textContent = `Total Lines: ${totalLines}`;
        // The polling should handle intermediate statuses.
        // Explicitly call once more to get final status if indexing was super quick
        await updateIndexingStatusDisplay(); 
        // If polling didn't stop it, and status is Ready, stop it.
        if (statusIndexing.textContent === "Ready" && indexingStatusInterval){
           clearInterval(indexingStatusInterval);
           indexingStatusInterval = null;
        }


        currentScrollTop = 0;
        rawViewContent.scrollTop = 0;
        rawViewLineNumbers.scrollTop = 0;
        if (lineHeight === 0) calculateLineHeight();
        if (lineHeight > 0) {
             updateVisibleLinesCount();
             await renderVisibleLines();
        } else {
            console.error("Cannot render: Line height is still 0.");
            statusIndexing.textContent = "Error: Line height unknown."; // Keep this specific error
        }

    } catch (error) { // Error from invoke('open_file')
        console.error("Error opening file:", error);
        statusFilePath.textContent = `Error: ${error}`;
        statusTotalLines.textContent = "Total Lines: 0";
        totalLines = 0;
        rawViewContent.innerHTML = '';
        rawViewLineNumbers.innerHTML = '';
        statusIndexing.textContent = `Error opening: ${error}`; // Show error
        if (indexingStatusInterval) {
            clearInterval(indexingStatusInterval);
            indexingStatusInterval = null;
        }
    }
}

// Listen for window resize events to recalculate visible lines
 appWindow.onFileDropEvent(async (event) => {
     if (event.payload.type === 'hover') {
         // Optionally add some UI feedback for hover
     } else if (event.payload.type === 'drop') {
         if (event.payload.paths.length > 0) {
             await handleFileOpen(event.payload.paths[0]); // Open the first dropped file
         }
     } else if (event.payload.type === 'cancel') {
         // User cancelled the drag
     }
 });


// --- Initialization ---
window.addEventListener('DOMContentLoaded', () => {
    calculateLineHeight();
    updateVisibleLinesCount();
    // Example: Try to open a file (for testing, replace with actual file open dialog logic)
    // This would typically be triggered by a File > Open menu item
    // For now, we can manually call open_file via dev console or a test button if needed.
    // Or, let's make the menu bar placeholder clickable for testing
    const fileMenu = document.querySelector('.menu-bar-placeholder span:first-child');
    if (fileMenu) {
        fileMenu.onclick = async () => {
             const { dialog } = window.__TAURI__;
             const filePath = await dialog.open({
                 multiple: false,
                 filters: [{ name: 'JSON Lines', extensions: ['jsonl', 'json'] }, { name: 'All Files', extensions: ['*'] }]
             });
             if (filePath && typeof filePath === 'string') {
                  await handleFileOpen(filePath);
             } else if (Array.isArray(filePath) && filePath.length > 0) {
                 await handleFileOpen(filePath[0]);
             }
        };
    }
});

window.addEventListener('resize', () => {
    if (lineHeight > 0) { // Only if lineHeight is known
        updateVisibleLinesCount();
        requestAnimationFrame(renderVisibleLines); // Re-render with new viewport size
    }
});
