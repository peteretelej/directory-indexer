use log::{debug, warn};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    error::{IndexerError, Result},
    utils::{chunk_text, detect_file_type, normalize_path, should_ignore_file, FileType},
};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub modified_time: u64,
    pub hash: String,
    pub parent_dirs: Vec<String>,
    pub content: Option<String>,
    pub errors: Option<String>,
}

pub struct FileScanner {
    max_file_size: u64,
    ignore_patterns: Vec<String>,
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB default
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ],
        }
    }

    pub fn with_ignore_patterns(ignore_patterns: Vec<String>) -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024,
            ignore_patterns,
        }
    }

    pub fn with_max_size(max_size: u64) -> Self {
        Self {
            max_file_size: max_size,
            ignore_patterns: vec![],
        }
    }

    pub async fn scan_directory(&self, dir_path: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(dir_path).follow_links(false) {
            let entry = entry.map_err(|e| {
                IndexerError::file_processing(format!("Error walking directory: {e}"))
            })?;

            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Apply ignore patterns
            if should_ignore_file(path, &self.ignore_patterns) {
                debug!("Ignoring file due to patterns: {path:?}");
                continue;
            }

            // Get file metadata
            let metadata = tokio::fs::metadata(path).await?;
            let size = metadata.len();

            let modified_time = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| IndexerError::file_processing(format!("Invalid modified time: {e}")))?
                .as_secs();

            // Calculate hash
            let hash = crate::utils::calculate_file_hash(path)?;

            // Extract parent directories
            let parent_dirs = self.extract_parent_directories(path, dir_path);

            // Check file size and read content if appropriate
            let (content, errors) = if size > self.max_file_size {
                (
                    None,
                    Some(format!(
                        "File too large: {size} bytes (max: {})",
                        self.max_file_size
                    )),
                )
            } else {
                match tokio::fs::read_to_string(path).await {
                    Ok(content) => (Some(content), None),
                    Err(e) => (None, Some(format!("Failed to read file: {e}"))),
                }
            };

            files.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                size,
                modified_time,
                hash,
                parent_dirs,
                content,
                errors,
            });
        }

        Ok(files)
    }

    fn extract_parent_directories(&self, file_path: &Path, root_dir: &Path) -> Vec<String> {
        let mut parent_dirs = Vec::new();

        // Add the root directory (normalized)
        if let Ok(normalized_root) = normalize_path(root_dir) {
            parent_dirs.push(normalized_root);
        }

        // Add all parent directories between root and file
        if let Ok(relative_path) = file_path.strip_prefix(root_dir) {
            let mut current = root_dir.to_path_buf();
            for component in relative_path.parent().unwrap_or(Path::new("")).components() {
                current = current.join(component);
                if let Ok(normalized_current) = normalize_path(&current) {
                    parent_dirs.push(normalized_current);
                }
            }
        }

        parent_dirs
    }
}

pub struct FileProcessor {
    max_file_size: u64,
    ignore_patterns: Vec<String>,
    chunk_size: usize,
    overlap: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub path: PathBuf,
    pub content: String,
    pub chunks: Vec<String>,
    pub file_type: Option<FileType>,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified_time: u64,
    pub file_type: Option<FileType>,
}

impl FileProcessor {
    pub fn new(
        max_file_size: u64,
        ignore_patterns: Vec<String>,
        chunk_size: usize,
        overlap: usize,
    ) -> Self {
        Self {
            max_file_size,
            ignore_patterns,
            chunk_size,
            overlap,
        }
    }

    pub async fn walk_directory(&self, dir_path: &Path) -> Result<Vec<FileMetadata>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(dir_path).follow_links(false) {
            let entry = entry.map_err(|e| {
                IndexerError::file_processing(format!("Error walking directory: {e}"))
            })?;

            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Apply ignore patterns
            if should_ignore_file(path, &self.ignore_patterns) {
                debug!("Ignoring file due to patterns: {path:?}");
                continue;
            }

            // Get file metadata
            let metadata = tokio::fs::metadata(path).await?;
            let size = metadata.len();

            // Skip files that are too large
            if size > self.max_file_size {
                warn!("Skipping large file ({size} bytes): {path:?}");
                continue;
            }

            let modified_time = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| IndexerError::file_processing(format!("Invalid modified time: {e}")))?
                .as_secs();

            let file_type = detect_file_type(path);

            files.push(FileMetadata {
                path: path.to_path_buf(),
                size,
                modified_time,
                file_type,
            });
        }

        Ok(files)
    }

    pub async fn process_file(&self, path: &Path) -> Result<ProcessedFile> {
        debug!("Processing file: {path:?}");

        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {e}")))?;

        // Chunk the content
        let chunks = chunk_text(&content, self.chunk_size, self.overlap);

        // Get file metadata
        let metadata = tokio::fs::metadata(path).await?;
        let size = metadata.len();
        let file_type = detect_file_type(path);

        // Calculate hash
        let hash = crate::utils::calculate_file_hash(path)?;

        Ok(ProcessedFile {
            path: path.to_path_buf(),
            content,
            chunks,
            file_type,
            size,
            hash,
        })
    }

    pub fn should_process_file(&self, file_type: &Option<FileType>) -> bool {
        // Only process text-based files for now
        match file_type {
            Some(FileType::Text)
            | Some(FileType::Code)
            | Some(FileType::Data)
            | Some(FileType::Markup)
            | Some(FileType::Config) => true,
            None => false,
        }
    }

    pub fn extract_parent_directories(
        &self,
        file_path: &Path,
        root_dirs: &[PathBuf],
    ) -> Vec<String> {
        let mut parent_dirs = Vec::new();

        for root in root_dirs {
            if let Ok(relative_path) = file_path.strip_prefix(root) {
                if let Some(parent) = relative_path.parent() {
                    if let Ok(normalized_parent) = normalize_path(root.join(parent)) {
                        parent_dirs.push(normalized_parent);
                    }
                }
                if let Ok(normalized_root) = normalize_path(root) {
                    parent_dirs.push(normalized_root);
                }
                break;
            }
        }

        parent_dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_walk_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test.txt"), "test content").unwrap();
        fs::write(temp_path.join("test.md"), "# Test").unwrap();

        let processor = FileProcessor::new(1024 * 1024, vec![], 512, 50);
        let files = processor.walk_directory(temp_path).await.unwrap();

        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_process_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let file_path = temp_path.join("test.txt");

        fs::write(
            &file_path,
            "This is a test file content that should be chunked.",
        )
        .unwrap();

        let processor = FileProcessor::new(1024 * 1024, vec![], 20, 5);
        let processed = processor.process_file(&file_path).await.unwrap();

        assert!(!processed.content.is_empty());
        assert!(!processed.chunks.is_empty());
        assert_eq!(processed.file_type, Some(FileType::Text));
    }
}
