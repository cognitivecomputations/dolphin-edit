use crate::indexing_service::NGRAM_SIZE;

pub fn generate_ngrams_from_line(line_content: &str) -> Vec<Vec<u8>> {
    let mut ngrams = Vec::new();
    let line_bytes = line_content.as_bytes();

    if line_bytes.len() < NGRAM_SIZE {
        // For now, let's only generate if length >= NGRAM_SIZE
        return ngrams;
    }

    for i in 0..=(line_bytes.len() - NGRAM_SIZE) {
        ngrams.push(line_bytes[i..i + NGRAM_SIZE].to_vec());
    }
    ngrams
}
