use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("directory-indexer"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("directory-indexer"));
}

#[test]
fn test_status_command() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Directory Indexer Status"));
}

#[test]
fn test_index_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file
    fs::write(temp_path.join("test.txt"), "This is a test file").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexing"));
}

#[test]
fn test_search_command() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("test query")
        .assert()
        .success()
        .stdout(predicate::str::contains("Searching"));
}

#[test]
fn test_similar_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let test_file = temp_path.join("test.txt");

    // Create a test file
    fs::write(&test_file, "This is a test file").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("similar")
        .arg(test_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Finding files similar"));
}

#[test]
fn test_get_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let test_file = temp_path.join("test.txt");

    // Create a test file
    fs::write(&test_file, "This is a test file").unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg(test_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Getting content"));
}

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_serve_command_help() {
        // Test that serve command accepts help flag
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("serve").arg("--help").assert().success();
    }

    #[test]
    fn test_qdrant_connection() {
        // This test requires Qdrant to be running
        // Skip if QDRANT_URL is not set
        if std::env::var("QDRANT_URL").is_err() {}

        // TODO: Add actual Qdrant connection test
        // This would test creating a collection, adding points, and searching
    }
}
