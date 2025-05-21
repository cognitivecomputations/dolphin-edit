use serde_json::Value;
use std::collections::HashSet;

fn extract_terms_from_value(json_value: &Value, terms: &mut HashSet<String>) {
    match json_value {
        Value::Object(map) => {
            for (key, value) in map {
                terms.insert(key.to_lowercase()); // Index keys
                extract_terms_from_value(value, terms);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                extract_terms_from_value(value, terms);
            }
        }
        Value::String(s) => {
            // Simple tokenization: split by whitespace and common punctuation.
            // More sophisticated tokenization can be added later.
            s.split_whitespace()
             .flat_map(|word| word.split(|c: char| !c.is_alphanumeric()))
             .filter(|word| !word.is_empty())
             .for_each(|term_part| {
                 terms.insert(term_part.to_lowercase()); // Index string parts
             });
        }
        Value::Number(n) => {
            terms.insert(n.to_string()); // Index numbers as strings
        }
        Value::Bool(b) => {
            terms.insert(b.to_string()); // Index booleans as strings ("true", "false")
        }
        Value::Null => {
            // Optionally index "null" or ignore
        }
    }
}

pub fn tokenize_json_line(line_content: &str) -> HashSet<String> {
    let mut terms = HashSet::new();
    if let Ok(value) = serde_json::from_str::<Value>(line_content) {
        extract_terms_from_value(&value, &mut terms);
    }
    // Else, if not valid JSON, no terms are extracted for this line.
    terms
}
