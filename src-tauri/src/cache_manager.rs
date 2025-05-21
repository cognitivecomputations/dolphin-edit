use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use super::indexing_service::{LineOffset, InvertedIndex, NGramIndex}; // Added NGramIndex
use std::error::Error; // For Box<dyn Error>

// Helper function to calculate a hash for a given value
fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// Helper function to determine the path for a cache file
fn get_cache_file_path(original_file_path: &str) -> Result<PathBuf, String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "DolphinEdit", "DolphinEdit") {
        let cache_dir = proj_dirs.cache_dir();
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir).map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }
        // Use the hash of the original file path to name the cache file
        let file_hash = calculate_hash(&original_file_path.to_string());
        Ok(cache_dir.join(format!("{}.indexcache", file_hash)))
    } else {
        Err("Could not determine project cache directory".to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct LineOffsetCacheWrapper {
    original_file_size: u64,
    original_mod_time_secs: u64,
    original_mod_time_nanos: u32,
    index_data: Vec<LineOffset>, // Adjusted to use the imported LineOffset
}

pub fn save_line_offset_index(file_path: &str, index_data: &Vec<LineOffset>) -> Result<(), Box<dyn Error>> {
    let cache_path = get_cache_file_path(file_path)?;
    let metadata = fs::metadata(file_path)?;
    let mod_time = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;

    let wrapper = LineOffsetCacheWrapper {
        original_file_size: metadata.len(),
        original_mod_time_secs: mod_time.as_secs(),
        original_mod_time_nanos: mod_time.subsec_nanos(),
        index_data: index_data.clone(),
    };

    let file = File::create(cache_path)?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, &wrapper)?;
    Ok(())
}

pub fn load_line_offset_index(file_path: &str) -> Result<Option<Vec<LineOffset>>, Box<dyn Error>> {
    let current_metadata = fs::metadata(file_path)?;
    let current_mod_time = current_metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;
    
    let cache_path = get_cache_file_path(file_path)?;

    if !cache_path.exists() {
        return Ok(None);
    }

    let file = File::open(cache_path)?;
    let reader = BufReader::new(file);
    let wrapper: LineOffsetCacheWrapper = bincode::deserialize_from(reader)?;

    if wrapper.original_file_size == current_metadata.len() &&
       wrapper.original_mod_time_secs == current_mod_time.as_secs() &&
       wrapper.original_mod_time_nanos == current_mod_time.subsec_nanos() {
        Ok(Some(wrapper.index_data))
    } else {
        Ok(None) // Cache is stale
    }
}

// --- NGram Index Cache ---

#[derive(Serialize, Deserialize)]
pub struct NGramIndexCacheWrapper {
    pub original_file_size: u64,
    pub original_mod_time_secs: u64,
    pub original_mod_time_nanos: u32,
    // pub ngram_size: usize, // If NGRAM_SIZE becomes dynamic, cache it too
    pub index_data: NGramIndex,
}

fn get_ngram_index_cache_file_path(original_file_path: &str) -> Result<PathBuf, String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "DolphinEdit", "DolphinEdit") {
        let cache_dir = proj_dirs.cache_dir();
        if !cache_dir.exists() {
            std::fs::create_dir_all(cache_dir).map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }
        let file_hash = calculate_hash(&original_file_path.to_string());
        Ok(cache_dir.join(format!("{}.ngram_index.cache", file_hash)))
    } else {
        Err("Could not determine project cache directory".to_string())
    }
}

pub fn save_ngram_index(file_path: &str, index_data: &NGramIndex) -> Result<(), Box<dyn Error>> {
    let cache_path = get_ngram_index_cache_file_path(file_path)?;
    let metadata = std::fs::metadata(file_path)?;
    let mod_time = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;

    let wrapper = NGramIndexCacheWrapper {
        original_file_size: metadata.len(),
        original_mod_time_secs: mod_time.as_secs(),
        original_mod_time_nanos: mod_time.subsec_nanos(),
        // ngram_size: crate::indexing_service::NGRAM_SIZE, // Store NGRAM_SIZE if it might change
        index_data: index_data.clone(),
    };

    let file = File::create(cache_path)?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, &wrapper)?;
    Ok(())
}

pub fn load_ngram_index(file_path: &str) -> Result<Option<NGramIndex>, Box<dyn Error>> {
    let cache_path = get_ngram_index_cache_file_path(file_path)?;
    if !cache_path.exists() {
        return Ok(None);
    }

    let current_metadata = std::fs::metadata(file_path)?;
    let current_mod_time = current_metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;

    let file = File::open(cache_path)?;
    let reader = BufReader::new(file);
    let wrapper: NGramIndexCacheWrapper = bincode::deserialize_from(reader)?;

    // Add check for ngram_size if it's stored in the cache wrapper
    // if wrapper.ngram_size != crate::indexing_service::NGRAM_SIZE { return Ok(None); }

    if wrapper.original_file_size == current_metadata.len() &&
       wrapper.original_mod_time_secs == current_mod_time.as_secs() &&
       wrapper.original_mod_time_nanos == current_mod_time.subsec_nanos() {
        Ok(Some(wrapper.index_data))
    } else {
        Ok(None) // Cache is stale
    }
}

// --- Inverted Index Cache ---

#[derive(Serialize, Deserialize)]
pub struct InvertedIndexCacheWrapper {
    pub original_file_size: u64,
    pub original_mod_time_secs: u64,
    pub original_mod_time_nanos: u32,
    pub index_data: InvertedIndex,
}

fn get_inverted_index_cache_file_path(original_file_path: &str) -> Result<PathBuf, String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "DolphinEdit", "DolphinEdit") {
        let cache_dir = proj_dirs.cache_dir();
        if !cache_dir.exists() {
            std::fs::create_dir_all(cache_dir).map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }
        let file_hash = calculate_hash(&original_file_path.to_string());
        Ok(cache_dir.join(format!("{}.inverted_index.cache", file_hash)))
    } else {
        Err("Could not determine project cache directory".to_string())
    }
}

pub fn save_inverted_index(file_path: &str, index_data: &InvertedIndex) -> Result<(), Box<dyn Error>> {
    let cache_path = get_inverted_index_cache_file_path(file_path)?;
    let metadata = std::fs::metadata(file_path)?;
    let mod_time = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;

    let wrapper = InvertedIndexCacheWrapper {
        original_file_size: metadata.len(),
        original_mod_time_secs: mod_time.as_secs(),
        original_mod_time_nanos: mod_time.subsec_nanos(),
        index_data: index_data.clone(),
    };

    let file = File::create(cache_path)?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, &wrapper)?;
    Ok(())
}

pub fn load_inverted_index(file_path: &str) -> Result<Option<InvertedIndex>, Box<dyn Error>> {
    let cache_path = get_inverted_index_cache_file_path(file_path)?;
    if !cache_path.exists() {
        return Ok(None);
    }

    let current_metadata = std::fs::metadata(file_path)?;
    let current_mod_time = current_metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;

    let file = File::open(cache_path)?;
    let reader = BufReader::new(file);
    let wrapper: InvertedIndexCacheWrapper = bincode::deserialize_from(reader)?;

    if wrapper.original_file_size == current_metadata.len() &&
       wrapper.original_mod_time_secs == current_mod_time.as_secs() &&
       wrapper.original_mod_time_nanos == current_mod_time.subsec_nanos() {
        Ok(Some(wrapper.index_data))
    } else {
        Ok(None) // Cache is stale
    }
}
