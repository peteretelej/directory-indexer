use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;
use serde_json::json;

mod fixtures;
use fixtures::create_test_files::TestDirectoryStructure;

/// Test handling of permission denied errors
#[test]
#[cfg(unix)]
fn test_permission_denied_directory() {
    let temp_dir = TempDir::new().unwrap();
    let restricted_dir = temp_dir.path().join("restricted");
    fs::create_dir(&restricted_dir).unwrap();
    
    // Create a file in the directory
    fs::write(restricted_dir.join("file.txt"), "test content").unwrap();
    
    // Remove read permissions from directory
    let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&restricted_dir, perms).unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(restricted_dir.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("permission").or(predicate::str::contains("access")));

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&restricted_dir, perms).unwrap();
}

/// Test handling of corrupted or unreadable files
#[test]
fn test_corrupted_files() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a file with binary content that might cause issues
    let binary_file = temp_dir.path().join("corrupted.bin");
    let corrupt_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x81, 0x82, 0x83];
    fs::write(&binary_file, corrupt_data).unwrap();
    
    // Create a very large file that exceeds limits
    let large_file = temp_dir.path().join("large.txt");
    let large_content = "x".repeat(50 * 1024 * 1024); // 50MB
    fs::write(&large_file, large_content).unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success(); // Should handle gracefully, not fail completely
}

/// Test handling of circular symlinks
#[test]
#[cfg(unix)]
fn test_circular_symlinks() {
    let temp_dir = TempDir::new().unwrap();
    let link1 = temp_dir.path().join("link1");
    let link2 = temp_dir.path().join("link2");
    
    // Create circular symlinks
    std::os::unix::fs::symlink(&link2, &link1).unwrap();
    std::os::unix::fs::symlink(&link1, &link2).unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success(); // Should handle gracefully without infinite loops
}

/// Test handling of very deep directory structures
#[test]
fn test_deep_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let mut current_path = temp_dir.path().to_path_buf();
    
    // Create a deeply nested directory structure (100 levels)
    for i in 0..100 {
        current_path = current_path.join(format!("level_{}", i));
        fs::create_dir_all(&current_path).unwrap();
    }
    
    // Create a file at the deepest level
    fs::write(current_path.join("deep_file.txt"), "Deep content").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

/// Test handling of files with special characters in names
#[test]
fn test_special_character_filenames() {
    let temp_dir = TempDir::new().unwrap();
    
    let special_filenames = vec![
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.with.dots.txt",
        "UPPERCASE.TXT",
        "MixedCase.TxT",
        "file@symbol.txt",
        "file#hash.txt",
        "file$dollar.txt",
        "file%percent.txt",
        "file&ampersand.txt",
        "file(paren).txt",
        "file[bracket].txt",
        "file{brace}.txt",
        "file+plus.txt",
        "file=equals.txt",
        "file~tilde.txt",
        "file`backtick.txt",
    ];

    for filename in special_filenames {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, format!("Content for {}", filename)).unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success();
}

/// Test handling of empty directories
#[test]
fn test_empty_directories() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create several empty directories
    for i in 0..10 {
        fs::create_dir(temp_dir.path().join(format!("empty_{}", i))).unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("indexed").or(predicate::str::contains("complete")));
}

/// Test handling of empty files
#[test]
fn test_empty_files() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create several empty files
    for i in 0..10 {
        fs::write(temp_dir.path().join(format!("empty_{}.txt", i)), "").unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success();
}

/// Test concurrent indexing of the same directory
#[test]
fn test_concurrent_indexing() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let handles: Vec<_> = (0..3).map(|_| {
        let path = test_path.to_string();
        std::thread::spawn(move || {
            let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
            cmd.arg("index")
                .arg(&path)
                .timeout(std::time::Duration::from_secs(30))
                .assert();
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test search with malformed queries
#[test]
fn test_malformed_search_queries() {
    let malformed_queries = vec![
        "\"unclosed quote",
        "query with\0null byte",
        "query with\ttab",
        "query with\nnewline",
        "\x01\x02\x03\x04", // Control characters
        "a".repeat(10000), // Very long query
        "", // Empty query
        "   ", // Only whitespace
        "query AND OR NOT", // Invalid boolean logic
        "query\"with\"embedded\"quotes",
    ];

    for query in malformed_queries {
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        let result = cmd.arg("search")
            .arg(query)
            .assert();
        
        // Should either succeed gracefully or fail with appropriate error
        // Don't crash or hang
    }
}

/// Test handling of network connectivity issues (for embedding providers)
#[test]
fn test_network_connectivity_issues() {
    // Test with invalid embedding provider endpoint
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bad_config.json");
    
    let bad_config = json!({
        "embedding": {
            "provider": "ollama",
            "endpoint": "http://localhost:99999", // Invalid port
            "model": "nomic-embed-text"
        }
    });
    
    fs::write(&config_path, serde_json::to_string_pretty(&bad_config).unwrap()).unwrap();

    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .failure() // Should fail gracefully with network error
        .stderr(predicate::str::contains("connection").or(predicate::str::contains("network")));
}

/// Test handling of invalid configuration files
#[test]
fn test_invalid_configuration_files() {
    let temp_dir = TempDir::new().unwrap();
    
    let invalid_configs = vec![
        ("empty.json", ""),
        ("invalid_json.json", "{ invalid json }"),
        ("missing_fields.json", r#"{"some": "field"}"#),
        ("wrong_types.json", r#"{"embedding": "not an object"}"#),
        ("null_values.json", r#"{"embedding": null}"#),
    ];

    for (filename, content) in invalid_configs {
        let config_path = temp_dir.path().join(filename);
        fs::write(&config_path, content).unwrap();

        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("--config")
            .arg(config_path.to_str().unwrap())
            .arg("status")
            .assert()
            .failure()
            .stderr(predicate::str::contains("config").or(predicate::str::contains("invalid")));
    }
}

/// Test resource exhaustion scenarios
#[test]
fn test_resource_exhaustion() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create many small files to test memory usage
    for i in 0..1000 {
        let content = format!("File {} content with some text to index", i);
        fs::write(temp_dir.path().join(format!("file_{}.txt", i)), content).unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success(); // Should handle many files without running out of resources
}

/// Test database corruption recovery
#[test]
fn test_database_corruption_handling() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("corrupted.db");
    
    // Create a corrupted database file
    fs::write(&db_path, "This is not a valid SQLite database").unwrap();

    let config_path = temp_dir.path().join("config.json");
    let config = json!({
        "storage": {
            "sqlite_path": db_path.to_str().unwrap(),
            "qdrant": {
                "endpoint": "http://localhost:6333",
                "collection": "test-collection"
            }
        }
    });
    
    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("database").or(predicate::str::contains("corrupt")));
}

/// Test handling of disk space exhaustion
#[test]
fn test_disk_space_handling() {
    // This test is challenging to implement without actually filling disk space
    // Instead, test with a read-only filesystem simulation
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test file
    fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();
    
    // Try to index to a location where we can't write (simulating disk full)
    let readonly_config = temp_dir.path().join("config.json");
    let config = json!({
        "storage": {
            "sqlite_path": "/dev/null/impossible.db", // This should fail
            "qdrant": {
                "endpoint": "http://localhost:6333",
                "collection": "test-collection"
            }
        }
    });
    
    fs::write(&readonly_config, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--config")
        .arg(readonly_config.to_str().unwrap())
        .arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("space").or(predicate::str::contains("write")));
}

/// Test handling of interrupted operations
#[test]
fn test_interrupted_operations() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    let mut child = cmd.arg("index")
        .arg(test_path)
        .spawn()
        .unwrap();

    // Let it run for a short time then kill it
    std::thread::sleep(std::time::Duration::from_millis(500));
    child.kill().unwrap();
    child.wait().unwrap();

    // Now try to index again - should handle partial state gracefully
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

/// Test handling of Unicode and international characters
#[test]
fn test_unicode_handling() {
    let temp_dir = TempDir::new().unwrap();
    
    let unicode_files = vec![
        ("chinese.txt", "这是中文测试文件。包含中文字符和标点符号。"),
        ("japanese.txt", "これは日本語のテストファイルです。漢字、ひらがな、カタカナが含まれています。"),
        ("arabic.txt", "هذا ملف اختبار باللغة العربية. يحتوي على نص عربي."),
        ("russian.txt", "Это тестовый файл на русском языке. Содержит кириллические символы."),
        ("emoji.txt", "This file contains emojis: 🚀 🎉 🔥 💻 📝 ✨ 🌟 ⭐"),
        ("mixed.txt", "Mixed: English, 中文, 日本語, العربية, Русский, 🌍"),
    ];

    for (filename, content) in unicode_files {
        fs::write(temp_dir.path().join(filename), content).unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success();

    // Test searching for Unicode content
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("中文")
        .assert()
        .success();
}

/// Test handling of very long file paths
#[test]
fn test_long_file_paths() {
    let temp_dir = TempDir::new().unwrap();
    let mut long_path = temp_dir.path().to_path_buf();
    
    // Create a path that's close to system limits
    let long_segment = "a".repeat(50);
    for _ in 0..10 {
        long_path = long_path.join(&long_segment);
        if let Err(_) = fs::create_dir_all(&long_path) {
            break; // Hit system limit
        }
    }
    
    // Create a file with a long name
    let long_filename = "b".repeat(100) + ".txt";
    let long_file = long_path.join(&long_filename);
    
    if fs::write(&long_file, "content").is_ok() {
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("index")
            .arg(temp_dir.path().to_str().unwrap())
            .assert()
            .success();
    }
}

/// Test error recovery and partial failures
#[test]
fn test_partial_failure_recovery() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a mix of good and problematic files
    fs::write(temp_dir.path().join("good1.txt"), "Good content 1").unwrap();
    fs::write(temp_dir.path().join("good2.txt"), "Good content 2").unwrap();
    
    // Create a file with problematic content
    let problematic_content = vec![0xFF; 1000]; // Binary content that might cause issues
    fs::write(temp_dir.path().join("problematic.bin"), problematic_content).unwrap();
    
    fs::write(temp_dir.path().join("good3.txt"), "Good content 3").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success(); // Should succeed despite problematic files

    // Verify we can still search and find the good files
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("Good content")
        .assert()
        .success();
}

/// Test MCP server error handling
#[test]
fn test_mcp_server_error_handling() {
    use std::process::{Command as StdCommand, Stdio};
    use std::io::Write;
    
    let mut server = StdCommand::new("cargo")
        .args(&["run", "--", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    let stdin = server.stdin.as_mut().unwrap();
    
    // Send malformed JSON
    let malformed_requests = vec![
        "not json at all",
        "{incomplete json",
        r#"{"jsonrpc": "2.0", "method": "nonexistent"}"#,
        r#"{"jsonrpc": "1.0", "method": "initialize"}"#, // Wrong version
        r#"{"method": "initialize"}"#, // Missing required fields
    ];

    for malformed in malformed_requests {
        let _ = stdin.write_all(malformed.as_bytes());
        let _ = stdin.write_all(b"\n");
        let _ = stdin.flush();
    }

    // Give server time to process
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let _ = server.kill();
    let _ = server.wait();
}