use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

mod fixtures;

/// Test handling of non-existent directories
#[test]
fn test_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg("/path/that/does/not/exist")
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

/// Test handling of empty directories
#[test]
fn test_empty_directories() {
    let temp_dir = TempDir::new().unwrap();

    // Create several empty directories
    for i in 0..5 {
        fs::create_dir(temp_dir.path().join(format!("empty_{}", i))).unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success(); // Should handle empty directories gracefully
}

/// Test handling of empty files
#[test]
fn test_empty_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create several empty files
    for i in 0..5 {
        fs::write(temp_dir.path().join(format!("empty_{}.txt", i)), "").unwrap();
    }

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success(); // Should handle empty files gracefully
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

/// Test search with very long query
#[test]
fn test_search_long_query() {
    let long_query = "a".repeat(1000);

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search").arg(&long_query).assert().success(); // Should handle long queries gracefully
}

/// Test similar files with non-existent file
#[test]
fn test_similar_nonexistent_file() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("similar")
        .arg("/path/to/nonexistent/file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

/// Test get content with non-existent file
#[test]
fn test_get_nonexistent_file() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg("/path/to/nonexistent/file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

/// Test handling of Unicode filenames and content
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

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success(); // Should handle Unicode gracefully
}

/// Test command with insufficient arguments
#[test]
fn test_insufficient_arguments() {
    // Test index command without path
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));

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
    cmd.arg("invalid-command").assert().failure().stderr(
        predicate::str::contains("unknown")
            .or(predicate::str::contains("invalid"))
            .or(predicate::str::contains("not found")),
    );
}

/// Test basic error recovery - should continue processing good files even with some bad ones
#[test]
fn test_partial_failure_recovery() {
    let temp_dir = TempDir::new().unwrap();

    // Create a mix of good and potentially problematic files
    fs::write(temp_dir.path().join("good1.txt"), "Good content 1").unwrap();
    fs::write(temp_dir.path().join("good2.txt"), "Good content 2").unwrap();

    // Create a file with binary content
    let binary_content = vec![0xFF; 100];
    fs::write(temp_dir.path().join("binary.dat"), binary_content).unwrap();

    fs::write(temp_dir.path().join("good3.txt"), "Good content 3").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .assert()
        .success(); // Should succeed despite problematic files
}
