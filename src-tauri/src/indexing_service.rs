use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;

pub type InvertedIndex = HashMap<String, Vec<u32>>;

pub const NGRAM_SIZE: usize = 3; 
pub type NGram = Vec<u8>;
pub type NGramIndex = HashMap<NGram, Vec<u32>>;

#[derive(Serialize, Deserialize, Debug, Clone)] 
pub struct LineOffset {
    pub offset: u64,
    pub length: usize,
}

pub fn build_line_offset_index(file_path: &str) -> Result<Vec<LineOffset>, std::io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut offsets = Vec::new();
    let mut current_offset = 0;
    let mut line_buffer = String::new();

    loop {
        let bytes_read = reader.read_line(&mut line_buffer)?;
        if bytes_read == 0 { // EOF
            break;
        }
        offsets.push(LineOffset { offset: current_offset, length: bytes_read });
        current_offset += bytes_read as u64;
        line_buffer.clear(); // Important to clear the buffer for the next read
    }
    Ok(offsets)
}

pub fn build_inverted_index(
    total_lines: usize,
    get_line_content_closure: &dyn Fn(u32) -> Option<String>
) -> Result<InvertedIndex, String> {
    let mut inverted_index = InvertedIndex::new();

    for line_num_u32 in 0..total_lines as u32 {
        if let Some(line_content) = get_line_content_closure(line_num_u32) {
            if line_content.trim().is_empty() { // Skip empty lines
                continue;
            }
            // Assuming token_utils is correctly brought into scope
            // e.g. use crate::utils::token_utils; or use super::utils::token_utils;
            let terms = crate::utils::token_utils::tokenize_json_line(&line_content);
            for term in terms {
                inverted_index.entry(term).or_default().push(line_num_u32);
            }
        } else {
            // Optionally log a warning or error if line content isn't available
            eprintln!("Warning: Could not retrieve content for line {}", line_num_u32);
        }
    }

    // Sort line number lists for efficient intersection later
    for postings_list in inverted_index.values_mut() { // Corrected Rpostings_list to postings_list
        postings_list.sort_unstable();
    }
    Ok(inverted_index)
}

pub fn build_ngram_index(
    total_lines: usize,
    get_line_content_closure: &dyn Fn(u32) -> Option<String>
) -> Result<NGramIndex, String> {
    let mut ngram_index = NGramIndex::new();

    for line_num_u32 in 0..total_lines as u32 {
        if let Some(line_content) = get_line_content_closure(line_num_u32) {
            if line_content.trim().is_empty() { // Skip empty lines
                continue;
            }
            let ngrams = crate::utils::ngram_utils::generate_ngrams_from_line(&line_content);
            for ngram_bytes in ngrams {
                let postings_list = ngram_index.entry(ngram_bytes).or_default();
                postings_list.push(line_num_u32);
            }
        } else {
            eprintln!("Warning: Could not retrieve content for N-gram indexing on line {}", line_num_u32);
        }
    }

    // Sort and deduplicate line number lists
    for postings_list in ngram_index.values_mut() {
        postings_list.sort_unstable();
        postings_list.dedup(); // N-grams can appear multiple times on the same line
    }
    Ok(ngram_index)
}
