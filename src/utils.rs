use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::error::{IndexerError, Result};

/// Convert a path to an absolute path
pub fn to_absolute_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(IndexerError::from)
            .map(|cwd| cwd.join(path))
    }
}

/// Calculate SHA256 hash of file content
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = std::fs::read(path)?;
    let hash = Sha256::digest(&content);
    Ok(format!("{:x}", hash))
}

/// Check if a file should be ignored based on patterns
pub fn should_ignore_file<P: AsRef<Path>>(path: P, ignore_patterns: &[String]) -> bool {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

    for pattern in ignore_patterns {
        // Check if pattern matches directory component
        if path_str.contains(pattern) {
            return true;
        }
        
        // Check for hidden files (starts with dot)
        if pattern == ".*" && file_name.starts_with('.') {
            return true;
        }
        
        // Check for files ending with pattern (like *~)
        if pattern.starts_with('*') && file_name.ends_with(&pattern[1..]) {
            return true;
        }
        
        // Direct file name match
        if file_name == *pattern {
            return true;
        }
    }
    false
}

/// Detect file type based on extension
pub fn detect_file_type<P: AsRef<Path>>(path: P) -> Option<FileType> {
    let extension = path.as_ref().extension()?.to_str()?.to_lowercase();

    match extension.as_str() {
        "md" | "txt" | "rst" | "org" => Some(FileType::Text),
        "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "h" => Some(FileType::Code),
        "json" | "yaml" | "yml" | "toml" | "csv" => Some(FileType::Data),
        "html" | "xml" => Some(FileType::Markup),
        "env" | "conf" | "ini" | "cfg" => Some(FileType::Config),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Text,
    Code,
    Data,
    Markup,
    Config,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Text => "text",
            FileType::Code => "code",
            FileType::Data => "data",
            FileType::Markup => "markup",
            FileType::Config => "config",
        }
    }
}

/// Split text into chunks with optional overlap
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = std::cmp::min(start + chunk_size, text.len());
        let chunk = text[start..end].to_string();
        chunks.push(chunk);

        if end == text.len() {
            break;
        }

        start = end.saturating_sub(overlap);
        if start == end.saturating_sub(overlap) && start > 0 {
            start = end;
        }
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let text = "This is a test text that should be chunked properly.";
        let chunks = chunk_text(text, 20, 5);

        assert!(!chunks.is_empty());
        assert!(chunks[0].len() <= 20);
    }

    #[test]
    fn test_file_type_detection() {
        assert_eq!(detect_file_type("test.md"), Some(FileType::Text));
        assert_eq!(detect_file_type("main.rs"), Some(FileType::Code));
        assert_eq!(detect_file_type("data.json"), Some(FileType::Data));
        assert_eq!(detect_file_type("unknown.xyz"), None);
    }

    #[test]
    fn test_should_ignore_file() {
        let patterns = vec![".git".to_string(), "node_modules".to_string()];

        assert!(should_ignore_file("path/.git/config", &patterns));
        assert!(should_ignore_file(
            "project/node_modules/package",
            &patterns
        ));
        assert!(!should_ignore_file("src/main.rs", &patterns));
    }
}
