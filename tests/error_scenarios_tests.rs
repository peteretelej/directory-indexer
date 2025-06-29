use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test handling of non-existent directories
#[test]
fn test_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg("/path/that/definitely/does/not/exist/12345")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

/// Test handling of files instead of directories
#[test]
fn test_file_instead_of_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("not_a_directory.txt");
    fs::write(&file_path, "This is a file, not a directory").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(file_path.to_str().unwrap())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not a directory")
                .or(predicate::str::contains("is not a directory")),
        );
}

/// Test handling of empty directories (without external services)
#[test]
fn test_empty_directories() {
    let temp_dir = TempDir::new().unwrap();

    // Create several empty directories
    for i in 0..3 {
        fs::create_dir(temp_dir.path().join(format!("empty_{}", i))).unwrap();
    }

    // Set environment to avoid connecting to external services
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.env("QDRANT_ENDPOINT", "http://localhost:9999") // Non-existent port
        .env("OLLAMA_ENDPOINT", "http://localhost:9998") // Non-existent port
        .arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .failure() // Should fail due to connectivity, but gracefully handle empty dirs
        .stderr(
            predicate::str::contains("connection")
                .or(predicate::str::contains("refused"))
                .or(predicate::str::contains("Environment setup")),
        );
}

/// Test handling of empty files (without external services)
#[test]
fn test_empty_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create several empty files
    for i in 0..3 {
        fs::write(temp_dir.path().join(format!("empty_{}.txt", i)), "").unwrap();
    }

    // Set environment to avoid connecting to external services
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.env("QDRANT_ENDPOINT", "http://localhost:9999") // Non-existent port
        .env("OLLAMA_ENDPOINT", "http://localhost:9998") // Non-existent port
        .arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .failure() // Should fail due to connectivity, but should handle empty files gracefully
        .stderr(
            predicate::str::contains("connection")
                .or(predicate::str::contains("refused"))
                .or(predicate::str::contains("Environment setup")),
        );
}

/// Test search with empty query
#[test]
fn test_search_empty_query() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty").or(predicate::str::contains("required")));
}

/// Test similar files with non-existent file
#[test]
fn test_similar_nonexistent_file() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("similar")
        .arg("/path/to/definitely/nonexistent/file12345.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

/// Test get content with non-existent file
#[test]
fn test_get_nonexistent_file() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg("/path/to/definitely/nonexistent/file12345.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

/// Test handling of Unicode filenames and content (without external services)
#[test]
fn test_unicode_handling() {
    let temp_dir = TempDir::new().unwrap();

    let unicode_files = vec![
        ("chinese.txt", "这是中文测试文件。"),
        ("japanese.txt", "これは日本語のテストファイルです。"),
        ("emoji.txt", "This file contains emojis: 🚀 🎉 🔥"),
    ];

    for (filename, content) in unicode_files {
        fs::write(temp_dir.path().join(filename), content).unwrap();
    }

    // Set environment to avoid connecting to external services
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.env("QDRANT_ENDPOINT", "http://localhost:9999") // Non-existent port
        .env("OLLAMA_ENDPOINT", "http://localhost:9998") // Non-existent port
        .arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .failure() // Should fail due to connectivity, but should handle Unicode gracefully
        .stderr(
            predicate::str::contains("connection")
                .or(predicate::str::contains("refused"))
                .or(predicate::str::contains("Environment setup")),
        );
}

/// Test command with insufficient arguments
#[test]
fn test_insufficient_arguments() {
    // Test search command without query
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));

    // Test similar command without file
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("similar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));

    // Test get command without file
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));
}

/// Test invalid command
#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("totally-invalid-command-12345")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unknown")
                .or(predicate::str::contains("invalid"))
                .or(predicate::str::contains("not found")),
        );
}

/// Test indexing with invalid chunk range
#[test]
fn test_invalid_chunk_range() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Some content for testing").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg(file_path.to_str().unwrap())
        .arg("--chunks")
        .arg("invalid-range")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("range")));
}

/// Test status command with invalid format
#[test]
fn test_status_invalid_format() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .arg("--format")
        .arg("invalid-format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("format")));
}
