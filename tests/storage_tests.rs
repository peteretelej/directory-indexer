use directory_indexer::storage::{FileRecord, SqliteStore};
use serde_json::json;
use tempfile::NamedTempFile;

#[cfg(test)]
mod sqlite_tests {
    use super::*;

    #[test]
    fn test_sqlite_store_creation_and_schema() {
        // Create a temporary database file
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path();

        // Create store - this should initialize schema
        let store = SqliteStore::new(db_path).expect("Failed to create SQLite store");

        // Verify tables exist by checking stats (this will fail if schema isn't created)
        let (dir_count, file_count, chunk_count) = store.get_stats().expect("Failed to get stats");
        assert_eq!(dir_count, 0);
        assert_eq!(file_count, 0);
        assert_eq!(chunk_count, 0);
    }

    #[test]
    fn test_directory_operations() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        // Test adding directory
        let dir_path = "/home/user/documents";
        let dir_id = store
            .add_directory(dir_path)
            .expect("Failed to add directory");
        assert!(dir_id > 0);

        // Test getting directories
        let directories = store.get_directories().expect("Failed to get directories");
        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0].path, dir_path);
        assert_eq!(directories[0].status, "pending");

        // Test updating directory status
        store
            .update_directory_status(dir_path, "completed")
            .expect("Failed to update status");
        let directories = store
            .get_directories()
            .expect("Failed to get directories after update");
        assert_eq!(directories[0].status, "completed");
    }

    #[test]
    fn test_file_operations() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        // Create test file record
        let file_record = FileRecord {
            id: 0, // Will be auto-assigned
            path: "/home/user/documents/test.md".to_string(),
            size: 1024,
            modified_time: 1234567890,
            hash: "abc123".to_string(),
            parent_dirs: vec!["/home/user/documents".to_string()],
            chunks_json: Some(json!([{"id": 0, "content": "test chunk"}])),
            errors_json: None,
        };

        // Test adding file
        let file_id = store.add_file(&file_record).expect("Failed to add file");
        assert!(file_id > 0);

        // Test getting file by path
        let retrieved_file = store
            .get_file_by_path(&file_record.path)
            .expect("Failed to get file")
            .expect("File not found");

        assert_eq!(retrieved_file.path, file_record.path);
        assert_eq!(retrieved_file.size, file_record.size);
        assert_eq!(retrieved_file.hash, file_record.hash);
        assert_eq!(retrieved_file.parent_dirs, file_record.parent_dirs);

        // Test file stats
        let (_, file_count, chunk_count) = store.get_stats().expect("Failed to get stats");
        assert_eq!(file_count, 1);
        assert_eq!(chunk_count, 1); // Has chunks_json
    }

    #[test]
    fn test_file_with_errors() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        let file_record = FileRecord {
            id: 0,
            path: "/home/user/error_file.txt".to_string(),
            size: 0,
            modified_time: 1234567890,
            hash: "".to_string(),
            parent_dirs: vec!["/home/user".to_string()],
            chunks_json: None,
            errors_json: Some(json!({"error": "File too large", "code": "FILE_SIZE_EXCEEDED"})),
        };

        store
            .add_file(&file_record)
            .expect("Failed to add file with errors");

        let retrieved_file = store
            .get_file_by_path(&file_record.path)
            .expect("Failed to get file")
            .expect("File not found");

        assert!(retrieved_file.errors_json.is_some());
        assert!(retrieved_file.chunks_json.is_none());

        let (_, file_count, chunk_count) = store.get_stats().expect("Failed to get stats");
        assert_eq!(file_count, 1);
        assert_eq!(chunk_count, 0); // No chunks_json
    }

    #[test]
    fn test_delete_file() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        let file_record = FileRecord {
            id: 0,
            path: "/home/user/temp.txt".to_string(),
            size: 100,
            modified_time: 1234567890,
            hash: "temp123".to_string(),
            parent_dirs: vec!["/home/user".to_string()],
            chunks_json: None,
            errors_json: None,
        };

        // Add file
        store.add_file(&file_record).expect("Failed to add file");

        // Verify it exists
        let file = store
            .get_file_by_path(&file_record.path)
            .expect("Failed to get file");
        assert!(file.is_some());

        // Delete file
        store
            .delete_file(&file_record.path)
            .expect("Failed to delete file");

        // Verify it's gone
        let file = store
            .get_file_by_path(&file_record.path)
            .expect("Failed to get file");
        assert!(file.is_none());
    }

    #[test]
    fn test_file_replacement() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        let file_path = "/home/user/update_test.txt";

        // First version
        let file_v1 = FileRecord {
            id: 0,
            path: file_path.to_string(),
            size: 100,
            modified_time: 1234567890,
            hash: "v1_hash".to_string(),
            parent_dirs: vec!["/home/user".to_string()],
            chunks_json: None,
            errors_json: None,
        };

        // Second version (updated)
        let file_v2 = FileRecord {
            id: 0,
            path: file_path.to_string(),
            size: 200,
            modified_time: 1234567900,
            hash: "v2_hash".to_string(),
            parent_dirs: vec!["/home/user".to_string()],
            chunks_json: Some(json!([{"id": 0, "content": "updated content"}])),
            errors_json: None,
        };

        // Add first version
        store.add_file(&file_v1).expect("Failed to add file v1");

        // Add second version (should replace)
        store.add_file(&file_v2).expect("Failed to add file v2");

        // Should only have one file with v2 data
        let retrieved = store
            .get_file_by_path(file_path)
            .expect("Failed to get file")
            .expect("File not found");

        assert_eq!(retrieved.hash, "v2_hash");
        assert_eq!(retrieved.size, 200);
        assert_eq!(retrieved.modified_time, 1234567900);

        let (_, file_count, _) = store.get_stats().expect("Failed to get stats");
        assert_eq!(file_count, 1); // Should still be just one file
    }
}

#[cfg(test)]
mod file_scanning_tests {
    use directory_indexer::indexing::files::FileScanner;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_scanning_and_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test files with known content
        let test_files = vec![
            ("document.md", "# Test Document\n\nThis is a markdown file."),
            ("config.json", r#"{"setting": "value"}"#),
            ("script.py", "def test():\n    return 'hello'"),
        ];

        let mut created_files = Vec::new();
        for (name, content) in &test_files {
            let file_path = base_path.join(name);
            fs::write(&file_path, content).unwrap();
            created_files.push(file_path);
        }

        // Create subdirectory with file
        let subdir = base_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let nested_file = subdir.join("nested.txt");
        fs::write(&nested_file, "nested content").unwrap();
        created_files.push(nested_file);

        // Test file scanning
        let scanner = FileScanner::new();
        let scanned_files = scanner
            .scan_directory(base_path)
            .await
            .expect("Failed to scan directory");

        // Should find all files
        assert_eq!(scanned_files.len(), 4);

        // Test metadata extraction
        for file_info in scanned_files {
            assert!(file_info.size > 0);
            assert!(file_info.modified_time > 0);
            assert!(!file_info.hash.is_empty());

            // Parent directories should be set correctly
            if file_info.path.contains("subdir") {
                assert!(file_info
                    .parent_dirs
                    .iter()
                    .any(|dir| dir.contains("subdir")));
            }
            assert!(file_info
                .parent_dirs
                .iter()
                .any(|dir| dir == base_path.to_str().unwrap()));
        }
    }

    #[tokio::test]
    async fn test_file_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create files including ones that should be ignored
        let files = vec!["valid.md", "also_valid.txt", ".hidden", "temp~"];

        for file in &files {
            fs::write(base_path.join(file), "content").unwrap();
        }

        // Create .git directory (should be ignored)
        let git_dir = base_path.join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "git config").unwrap();

        let ignore_patterns = vec![".git".to_string(), ".*".to_string(), "*~".to_string()];
        let scanner = FileScanner::with_ignore_patterns(ignore_patterns);
        let scanned_files = scanner
            .scan_directory(base_path)
            .await
            .expect("Failed to scan directory");

        // Should only find the valid files
        assert_eq!(scanned_files.len(), 2);
        let paths: Vec<String> = scanned_files
            .iter()
            .map(|f| {
                std::path::Path::new(&f.path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert!(paths.contains(&"valid.md".to_string()));
        assert!(paths.contains(&"also_valid.txt".to_string()));
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.txt");

        // Create a large file (simulate by setting a small max_file_size)
        let content = "x".repeat(1000); // 1KB file
        fs::write(&large_file, content).unwrap();

        let scanner = FileScanner::with_max_size(500); // 500 bytes max
        let scanned_files = scanner
            .scan_directory(temp_dir.path())
            .await
            .expect("Failed to scan directory");

        // File should be scanned but marked as too large
        assert_eq!(scanned_files.len(), 1);
        let file_info = &scanned_files[0];
        assert!(file_info
            .errors
            .as_ref()
            .is_some_and(|e| e.contains("too large")));
    }
}

#[cfg(test)]
mod metadata_storage_tests {
    use super::*;
    use directory_indexer::indexing::files::FileInfo;

    #[test]
    fn test_store_file_metadata_before_chunking() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        // Simulate file scanning result
        let file_info = FileInfo {
            path: "/test/document.md".to_string(),
            size: 1024,
            modified_time: 1234567890,
            hash: "file_hash_123".to_string(),
            parent_dirs: vec!["/test".to_string()],
            content: Some("# Document\n\nContent here".to_string()),
            errors: None,
        };

        // Store metadata before chunking (no chunks_json yet)
        let file_record = FileRecord {
            id: 0,
            path: file_info.path.clone(),
            size: file_info.size as i64,
            modified_time: file_info.modified_time as i64,
            hash: file_info.hash.clone(),
            parent_dirs: file_info.parent_dirs.clone(),
            chunks_json: None, // Not chunked yet
            errors_json: None,
        };

        let file_id = store
            .add_file(&file_record)
            .expect("Failed to store file metadata");
        assert!(file_id > 0);

        // Verify stored metadata
        let stored = store
            .get_file_by_path(&file_info.path)
            .expect("Failed to retrieve file")
            .expect("File not found");

        assert_eq!(stored.path, file_info.path);
        assert_eq!(stored.size, file_info.size as i64);
        assert_eq!(stored.hash, file_info.hash);
        assert_eq!(stored.parent_dirs, file_info.parent_dirs);
        assert!(stored.chunks_json.is_none()); // No chunks yet
        assert!(stored.errors_json.is_none()); // No errors
    }

    #[test]
    fn test_store_file_with_processing_errors() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        // Simulate file with processing error
        let file_info = FileInfo {
            path: "/test/binary.exe".to_string(),
            size: 5 * 1024 * 1024, // 5MB
            modified_time: 1234567890,
            hash: "binary_hash".to_string(),
            parent_dirs: vec!["/test".to_string()],
            content: None,
            errors: Some("File too large for processing".to_string()),
        };

        let file_record = FileRecord {
            id: 0,
            path: file_info.path.clone(),
            size: file_info.size as i64,
            modified_time: file_info.modified_time as i64,
            hash: file_info.hash.clone(),
            parent_dirs: file_info.parent_dirs.clone(),
            chunks_json: None,
            errors_json: Some(json!({
                "error": file_info.errors.as_ref().unwrap(),
                "timestamp": 1234567890,
                "stage": "file_reading"
            })),
        };

        store
            .add_file(&file_record)
            .expect("Failed to store file with errors");

        let stored = store
            .get_file_by_path(&file_info.path)
            .expect("Failed to retrieve file")
            .expect("File not found");

        assert!(stored.errors_json.is_some());
        let errors = stored.errors_json.unwrap();
        assert!(errors["error"].as_str().unwrap().contains("too large"));
    }

    #[test]
    fn test_update_file_with_chunks() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        let file_path = "/test/document.md";

        // First store without chunks
        let initial_record = FileRecord {
            id: 0,
            path: file_path.to_string(),
            size: 1024,
            modified_time: 1234567890,
            hash: "hash123".to_string(),
            parent_dirs: vec!["/test".to_string()],
            chunks_json: None,
            errors_json: None,
        };

        store
            .add_file(&initial_record)
            .expect("Failed to store initial file");

        // Update with chunks after processing
        let chunks = json!([
            {
                "id": 0,
                "start": 0,
                "end": 100,
                "content": "First chunk content",
                "token_count": 25
            },
            {
                "id": 1,
                "start": 80,
                "end": 200,
                "content": "Second chunk with overlap",
                "token_count": 30
            }
        ]);

        let updated_record = FileRecord {
            id: 0,
            path: file_path.to_string(),
            size: 1024,
            modified_time: 1234567890,
            hash: "hash123".to_string(),
            parent_dirs: vec!["/test".to_string()],
            chunks_json: Some(chunks.clone()),
            errors_json: None,
        };

        store
            .add_file(&updated_record)
            .expect("Failed to update file with chunks");

        // Verify chunks were stored
        let stored = store
            .get_file_by_path(file_path)
            .expect("Failed to retrieve file")
            .expect("File not found");

        assert!(stored.chunks_json.is_some());
        let stored_chunks = stored.chunks_json.unwrap();
        assert_eq!(stored_chunks, chunks);

        // Should show up in chunk count
        let (_, _, chunk_count) = store.get_stats().expect("Failed to get stats");
        assert_eq!(chunk_count, 1);
    }

    #[test]
    fn test_batch_file_storage() {
        let temp_db = NamedTempFile::new().unwrap();
        let store = SqliteStore::new(temp_db.path()).unwrap();

        // Simulate batch of files from directory scan
        let files = vec![
            ("file1.md", 500, "hash1"),
            ("file2.txt", 750, "hash2"),
            ("file3.json", 200, "hash3"),
        ];

        for (i, (name, size, hash)) in files.iter().enumerate() {
            let record = FileRecord {
                id: 0,
                path: format!("/batch/test/{}", name),
                size: *size,
                modified_time: 1234567890 + i as i64,
                hash: hash.to_string(),
                parent_dirs: vec!["/batch/test".to_string()],
                chunks_json: None,
                errors_json: None,
            };

            store.add_file(&record).expect("Failed to store batch file");
        }

        let (_, file_count, _) = store.get_stats().expect("Failed to get stats");
        assert_eq!(file_count, 3);

        // Verify all files stored correctly
        for (name, size, hash) in files {
            let path = format!("/batch/test/{}", name);
            let stored = store
                .get_file_by_path(&path)
                .expect("Failed to retrieve file")
                .expect("File not found");

            assert_eq!(stored.size, size);
            assert_eq!(stored.hash, hash);
        }
    }
}
