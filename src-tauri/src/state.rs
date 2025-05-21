use std::sync::Mutex;
use super::indexing_service::{LineOffset, InvertedIndex, NGramIndex}; // Add InvertedIndex, NGramIndex

pub struct AppState {
    pub current_file_path: Mutex<Option<String>>,
    pub line_offset_index: Mutex<Option<Vec<LineOffset>>>,
    pub inverted_index: Mutex<Option<InvertedIndex>>, // New
    pub ngram_index: Mutex<Option<NGramIndex>>,       // New
    pub indexing_status_message: Mutex<String>,      // New
    pub indexing_progress: Mutex<f32>,             // New
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            current_file_path: Mutex::new(None),
            line_offset_index: Mutex::new(None),
            inverted_index: Mutex::new(None),    // New
            ngram_index: Mutex::new(None),       // New
            indexing_status_message: Mutex::new("Ready".to_string()), // New
            indexing_progress: Mutex::new(0.0), // New
        }
    }
}
