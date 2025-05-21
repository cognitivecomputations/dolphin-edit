use tauri::State;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc; // For sharing line_offset_index in closure
use serde::Serialize; // For the IndexingStatus struct

use super::state::AppState;
use super::indexing_service::{self, LineOffset}; // Ensure LineOffset is in scope
use super::cache_manager;


#[derive(Clone, Serialize)] // Serialize for sending to frontend
pub struct IndexingStatus {
    message: String,
    progress: f32,
}

#[tauri::command]
pub fn open_file(file_path: String, app_state: State<AppState>) -> Result<usize, String> {
    // Helper to update status
    let set_status = |msg: &str, progress: f32, state: &State<AppState>| {
        match state.indexing_status_message.lock() {
            Ok(mut lock) => *lock = msg.to_string(),
            Err(e) => eprintln!("Failed to lock indexing_status_message: {}",e),
        }
        match state.indexing_progress.lock() {
            Ok(mut lock) => *lock = progress,
            Err(e) => eprintln!("Failed to lock indexing_progress: {}",e),
        }
        // Optionally, emit an event to the frontend immediately if desired
        // For simplicity, we'll rely on the frontend polling get_indexing_status for now.
        // state.window().emit("indexing_status_update", IndexingStatus { message: msg.to_string(), progress }).unwrap_or_else(|e| {
        //     eprintln!("Failed to emit indexing_status_update: {}", e);
        // });
    };

    set_status("Opening file...", 0.0, &app_state);

    // 1. Reset state for new file
    *app_state.current_file_path.lock().map_err(|e| format!("Failed to lock current_file_path: {}", e))? = Some(file_path.clone());
    *app_state.line_offset_index.lock().map_err(|e| format!("Failed to lock line_offset_index: {}", e))? = None;
    *app_state.inverted_index.lock().map_err(|e| format!("Failed to lock inverted_index: {}", e))? = None;
    *app_state.ngram_index.lock().map_err(|e| format!("Failed to lock ngram_index: {}", e))? = None;
    
    let total_lines_count: usize;

    // 2. Line Offset Index
    set_status("Indexing line offsets...", 0.1, &app_state);
    let line_offset_index_arc = match cache_manager::load_line_offset_index(&file_path) {
        Ok(Some(cached_index)) => {
            set_status("Loaded line offsets from cache.", 0.25, &app_state);
            Arc::new(cached_index)
        }
        _ => { // Cache miss or error
            match indexing_service::build_line_offset_index(&file_path) {
                Ok(built_index) => {
                    if let Err(e) = cache_manager::save_line_offset_index(&file_path, &built_index) {
                        eprintln!("Failed to save line offset index to cache: {}", e);
                    }
                    set_status("Built line offsets.", 0.25, &app_state);
                    Arc::new(built_index)
                }
                Err(e) => {
                    set_status(&format!("Error: {}", e), 0.0, &app_state);
                    return Err(format!("Failed to build line offset index: {}", e));
                }
            }
        }
    };
    total_lines_count = line_offset_index_arc.len();
    *app_state.line_offset_index.lock().map_err(|e| format!("Failed to lock line_offset_index: {}", e))? = Some(line_offset_index_arc.as_ref().clone());

    let loi_clone_for_closure = Arc::clone(&line_offset_index_arc);
    let file_path_clone_for_closure = Arc::new(file_path.clone());

    let get_line_content_closure = move |line_num_u32: u32| -> Option<String> {
        if (line_num_u32 as usize) < loi_clone_for_closure.len() {
            let line_info = &loi_clone_for_closure[line_num_u32 as usize]; 
            
            let mut file = match std::fs::File::open(file_path_clone_for_closure.as_ref()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Closure: Failed to open file {:?}: {}", file_path_clone_for_closure, e);
                    return None;
                }
            };
            let mut buffer = vec![0; line_info.length];
            
            if let Err(e) = file.seek(SeekFrom::Start(line_info.offset)) {
                eprintln!("Closure: Failed to seek in file: {}", e);
                return None;
            }
            if let Err(e) = file.read_exact(&mut buffer) {
                eprintln!("Closure: Failed to read line from file: {}", e);
                return None;
            }
            String::from_utf8(buffer).ok()
        } else {
            eprintln!("Closure: Line number {} out of bounds (len: {})", line_num_u32, loi_clone_for_closure.len());
            None
        }
    };
    
    // 3. Inverted Index
    set_status("Building inverted index...", 0.3, &app_state);
    match cache_manager::load_inverted_index(&file_path) {
        Ok(Some(cached_index)) => {
            *app_state.inverted_index.lock().map_err(|e| format!("Failed to lock inverted_index: {}", e))? = Some(cached_index);
            set_status("Loaded inverted index from cache.", 0.6, &app_state);
        }
        _ => {
            match indexing_service::build_inverted_index(total_lines_count, &get_line_content_closure) {
                Ok(built_index) => {
                    if let Err(e) = cache_manager::save_inverted_index(&file_path, &built_index) {
                        eprintln!("Failed to save inverted index to cache: {}", e);
                    }
                    *app_state.inverted_index.lock().map_err(|e| format!("Failed to lock inverted_index: {}", e))? = Some(built_index);
                    set_status("Built inverted index.", 0.6, &app_state);
                }
                Err(e) => {
                    eprintln!("Failed to build inverted index: {}. Proceeding without it.", e);
                    set_status(&format!("Failed to build inverted index: {}. Some search features may be unavailable.", e), 0.6, &app_state);
                }
            }
        }
    }

    // 4. N-gram Index
    set_status("Building N-gram index...", 0.65, &app_state);
    match cache_manager::load_ngram_index(&file_path) {
        Ok(Some(cached_index)) => {
            *app_state.ngram_index.lock().map_err(|e| format!("Failed to lock ngram_index: {}", e))? = Some(cached_index);
            set_status("Loaded N-gram index from cache.", 0.95, &app_state);
        }
        _ => {
            match indexing_service::build_ngram_index(total_lines_count, &get_line_content_closure) {
                Ok(built_index) => {
                    if let Err(e) = cache_manager::save_ngram_index(&file_path, &built_index) {
                        eprintln!("Failed to save N-gram index to cache: {}", e);
                    }
                    *app_state.ngram_index.lock().map_err(|e| format!("Failed to lock ngram_index: {}", e))? = Some(built_index);
                    set_status("Built N-gram index.", 0.95, &app_state);
                }
                Err(e) => {
                    eprintln!("Failed to build N-gram index: {}. Proceeding without it.", e);
                    set_status(&format!("Failed to build N-gram index: {}. Some search features may be unavailable.", e), 0.95, &app_state);
                }
            }
        }
    }

    set_status("Ready", 1.0, &app_state);
    Ok(total_lines_count)
}


#[tauri::command]
pub fn get_total_lines(app_state: State<AppState>) -> Result<usize, String> {
    let index_lock = app_state.line_offset_index.lock().map_err(|e| format!("Failed to lock index state: {}", e))?;
    match &*index_lock {
        Some(index) => Ok(index.len()),
        None => Ok(0),
    }
}

#[tauri::command]
pub fn get_lines(start_line: usize, count: usize, app_state: State<AppState>) -> Result<Vec<String>, String> {
    let file_path_lock = app_state.current_file_path.lock().map_err(|e| format!("Failed to lock file path state: {}", e))?;
    let current_file_path = match &*file_path_lock {
        Some(path) => path.clone(),
        None => return Err("No file is currently open.".to_string()),
    };

    let index_lock = app_state.line_offset_index.lock().map_err(|e| format!("Failed to lock index state: {}", e))?;
    let line_offsets = match &*index_lock {
        Some(index) => index,
        None => return Err("Line offset index is not available.".to_string()),
    };

    if line_offsets.is_empty() {
        return Ok(Vec::new());
    }

    let mut lines = Vec::with_capacity(count);
    let end_line = std::cmp::min(start_line + count, line_offsets.len());

    if start_line >= end_line {
         return Ok(Vec::new());
    }
    
    let mut file = File::open(&current_file_path)
        .map_err(|e| format!("Failed to open file {}: {}", current_file_path, e))?;

    for i in start_line..end_line {
        if let Some(line_offset_info) = line_offsets.get(i) {
            let mut buffer = vec![0; line_offset_info.length]; 
            file.seek(SeekFrom::Start(line_offset_info.offset))
                .map_err(|e| format!("Failed to seek in file: {}", e))?;
            file.read_exact(&mut buffer)
                .map_err(|e| format!("Failed to read line {} from file: {}", i, e))?;
            
            let line_content = String::from_utf8(buffer)
                .map_err(|e| format!("Failed to decode line {} as UTF-8: {}", i, e))?;
            lines.push(line_content);
        } else {
            return Err(format!("Internal error: Attempted to access line index {} which is out of bounds after check.", i));
        }
    }
    Ok(lines)
}

#[tauri::command]
pub fn get_line_content(line_number: usize, app_state: State<AppState>) -> Result<String, String> {
    let file_path_lock = app_state.current_file_path.lock().map_err(|e| format!("Failed to lock file path state: {}", e))?;
    let current_file_path = match &*file_path_lock {
        Some(path) => path.clone(),
        None => return Err("No file is currently open.".to_string()),
    };

    let index_lock = app_state.line_offset_index.lock().map_err(|e| format!("Failed to lock index state: {}", e))?;
    let line_offset_info = match &*index_lock {
        Some(index) => {
            if line_number >= index.len() {
                return Err(format!("Line number {} is out of bounds. Total lines: {}", line_number, index.len()));
            }
            index.get(line_number)
        }
        None => return Err("Line offset index is not available.".to_string()),
    };

    if let Some(info) = line_offset_info {
        let mut file = File::open(&current_file_path)
            .map_err(|e| format!("Failed to open file {}: {}", current_file_path, e))?;
        
        let mut buffer = vec![0; info.length];
        file.seek(SeekFrom::Start(info.offset))
            .map_err(|e| format!("Failed to seek in file: {}", e))?;
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Failed to read line {} from file: {}", line_number, e))?;
        
        String::from_utf8(buffer)
            .map_err(|e| format!("Failed to decode line {} as UTF-8: {}", line_number, e))
    } else {
        Err(format!("Could not retrieve line offset info for line {}.", line_number))
    }
}

#[tauri::command]
pub fn get_indexing_status(app_state: State<AppState>) -> Result<IndexingStatus, String> {
    let message = app_state.indexing_status_message.lock().map_err(|e| format!("Failed to lock indexing_status_message: {}", e))?.clone();
    let progress = *app_state.indexing_progress.lock().map_err(|e| format!("Failed to lock indexing_progress: {}", e))?;
    Ok(IndexingStatus { message, progress })
}
