use log::{debug, warn};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    error::{IndexerError, Result},
    utils::{chunk_text, detect_file_type, should_ignore_file, FileType},
};

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

    pub fn walk_directory(&self, dir_path: &Path) -> Result<Vec<FileMetadata>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(dir_path).follow_links(false) {
            let entry = entry.map_err(|e| {
                IndexerError::file_processing(format!("Error walking directory: {}", e))
            })?;

            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Apply ignore patterns
            if should_ignore_file(path, &self.ignore_patterns) {
                debug!("Ignoring file due to patterns: {:?}", path);
                continue;
            }

            // Get file metadata
            let metadata = std::fs::metadata(path)?;
            let size = metadata.len();

            // Skip files that are too large
            if size > self.max_file_size {
                warn!("Skipping large file ({} bytes): {:?}", size, path);
                continue;
            }

            let modified_time = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| {
                    IndexerError::file_processing(format!("Invalid modified time: {}", e))
                })?
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

    pub fn process_file(&self, path: &Path) -> Result<ProcessedFile> {
        debug!("Processing file: {:?}", path);

        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {}", e)))?;

        // Chunk the content
        let chunks = chunk_text(&content, self.chunk_size, self.overlap);

        // Get file metadata
        let metadata = std::fs::metadata(path)?;
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
                    parent_dirs.push(parent.to_string_lossy().to_string());
                }
                parent_dirs.push(root.to_string_lossy().to_string());
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

    #[test]
    fn test_walk_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test.txt"), "test content").unwrap();
        fs::write(temp_path.join("test.md"), "# Test").unwrap();

        let processor = FileProcessor::new(1024 * 1024, vec![], 512, 50);
        let files = processor.walk_directory(temp_path).unwrap();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_process_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let file_path = temp_path.join("test.txt");

        fs::write(
            &file_path,
            "This is a test file content that should be chunked.",
        )
        .unwrap();

        let processor = FileProcessor::new(1024 * 1024, vec![], 20, 5);
        let processed = processor.process_file(&file_path).unwrap();

        assert!(!processed.content.is_empty());
        assert!(!processed.chunks.is_empty());
        assert_eq!(processed.file_type, Some(FileType::Text));
    }
}
