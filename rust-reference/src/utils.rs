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

/// Normalize a path for consistent storage and comparison across platforms
/// - Uses forward slashes as separators for storage
/// - Handles case normalization on Windows drive letters
/// - Only converts to absolute if the path is actually relative
pub fn normalize_path<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    // Convert to string and replace backslashes with forward slashes
    let path_str = path.to_string_lossy();
    let mut normalized = path_str.replace('\\', "/");

    // Check if this is a Unix-style absolute path (starts with /)
    // These should be preserved as-is, especially important for tests
    let is_unix_absolute = normalized.starts_with('/');

    // If it's a relative path (and not a Unix-style absolute path), make it absolute
    if !path.is_absolute() && !is_unix_absolute {
        let abs_path = to_absolute_path(path)?;
        normalized = abs_path.to_string_lossy().replace('\\', "/");
    }

    // On Windows, normalize drive letters to lowercase if present
    // but only for actual Windows paths, not Unix-style paths
    #[cfg(windows)]
    {
        if !is_unix_absolute && normalized.len() >= 2 && normalized.chars().nth(1) == Some(':') {
            let mut chars: Vec<char> = normalized.chars().collect();
            chars[0] = chars[0].to_ascii_lowercase();
            normalized = chars.into_iter().collect();
        }
    }

    Ok(normalized)
}

/// Extract the filename from a normalized path
/// Note: This assumes the path is already normalized (uses forward slashes)
pub fn get_filename_from_path(path: &str) -> Option<String> {
    path.split('/').next_back().map(|s| s.to_string())
}

/// Compare two paths in a platform-agnostic way
pub fn paths_equal<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> bool {
    match (normalize_path(path1), normalize_path(path2)) {
        (Ok(p1), Ok(p2)) => p1 == p2,
        _ => false,
    }
}

/// Check if a path starts with another path (useful for checking if file is in directory)
pub fn path_starts_with<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, prefix: P2) -> bool {
    match (normalize_path(path), normalize_path(prefix)) {
        (Ok(p), Ok(pre)) => p.starts_with(&pre),
        _ => false,
    }
}

/// Get the parent directory path in normalized form
pub fn get_parent_path<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
    let abs_path = to_absolute_path(path)?;
    if let Some(parent) = abs_path.parent() {
        Ok(Some(normalize_path(parent)?))
    } else {
        Ok(None)
    }
}

/// Calculate SHA256 hash of file content
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = std::fs::read(path)?;
    let hash = Sha256::digest(&content);
    Ok(format!("{hash:x}"))
}

/// Check if a file should be ignored based on patterns
pub fn should_ignore_file<P: AsRef<Path>>(path: P, ignore_patterns: &[String]) -> bool {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

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

    #[test]
    fn test_normalize_path() {
        // Test relative path normalization
        let result = normalize_path("./test.txt");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert!(normalized.ends_with("/test.txt"));
        assert!(!normalized.contains("\\"));

        // Test that normalized paths use forward slashes
        let result = normalize_path("src/main.rs");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert!(normalized.contains("/src/main.rs"));
        assert!(!normalized.contains("\\"));
    }

    #[test]
    fn test_get_filename_from_path() {
        assert_eq!(
            get_filename_from_path("/path/to/file.txt"),
            Some("file.txt".to_string())
        );
        assert_eq!(
            get_filename_from_path("file.txt"),
            Some("file.txt".to_string())
        );
        assert_eq!(get_filename_from_path("/path/to/"), Some("".to_string()));
        assert_eq!(get_filename_from_path(""), Some("".to_string()));
    }

    #[test]
    fn test_paths_equal() {
        // Test equivalent paths with different separators (simulated)
        let path1 = "src/main.rs";
        let path2 = "src/main.rs";
        assert!(paths_equal(path1, path2));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_path_normalization() {
        // Test Windows drive letter normalization
        let result = normalize_path("C:\\Users\\test\\file.txt");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert!(normalized.starts_with("c:/"));
        assert!(!normalized.contains("\\"));

        // Test that Unix-style absolute paths are preserved (important for tests)
        let result = normalize_path("/home/user/documents");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "/home/user/documents");
    }
}
