use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

mod fixtures;
use fixtures::create_test_files::TestDirectoryStructure;

/// Test the `index` command with various directory structures
#[test]
fn test_index_command_comprehensive() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexing").or(predicate::str::contains("indexed")));
}

/// Test indexing multiple directories
#[test]
fn test_index_multiple_directories() {
    let test_structure1 = TestDirectoryStructure::new();
    let test_structure2 = TestDirectoryStructure::new();
    
    let path1 = test_structure1.path().to_str().unwrap();
    let path2 = test_structure2.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(path1)
        .arg(path2)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();
}

/// Test indexing non-existent directory
#[test]
fn test_index_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg("/path/that/does/not/exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

/// Test indexing with relative paths
#[test]
fn test_index_relative_path() {
    let test_structure = TestDirectoryStructure::new();
    let current_dir = std::env::current_dir().unwrap();
    
    // Change to the test directory parent
    std::env::set_current_dir(test_structure.path().parent().unwrap()).unwrap();
    
    let relative_path = test_structure.path().file_name().unwrap().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(relative_path)
        .assert()
        .success();
    
    // Restore original directory
    std::env::set_current_dir(current_dir).unwrap();
}

/// Test search command with various queries
#[test]
fn test_search_command_comprehensive() {
    let test_queries = vec![
        "database connection",
        "search engine",
        "performance optimization",
        "error handling",
        "rust programming",
        "configuration settings",
        "API endpoints",
    ];

    for query in test_queries {
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("search")
            .arg(query)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
            .success()
            .stdout(predicate::str::contains("Search").or(predicate::str::contains("results")));
    }
}

/// Test search with directory scope
#[test]
fn test_search_with_directory_scope() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("database")
        .arg("--path")
        .arg(test_path)
        .assert()
        .success();
}

/// Test search with limit parameter
#[test]
fn test_search_with_limit() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("test")
        .arg("--limit")
        .arg("5")
        .assert()
        .success();
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

/// Test similar files command with various file types
#[test]
fn test_similar_command_comprehensive() {
    let test_structure = TestDirectoryStructure::new();
    let test_files = vec![
        "docs/README.md",
        "src/main.rs",
        "config.json",
        "data/users.csv",
        "scripts/setup.sh",
    ];

    for file_path in test_files {
        let full_path = test_structure.path().join(file_path);
        if full_path.exists() {
            let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
            cmd.arg("similar")
                .arg(full_path.to_str().unwrap())
                .timeout(std::time::Duration::from_secs(15))
                .assert()
                .success()
                .stdout(predicate::str::contains("similar").or(predicate::str::contains("files")));
        }
    }
}

/// Test similar files with limit
#[test]
fn test_similar_with_limit() {
    let test_structure = TestDirectoryStructure::new();
    let readme_path = test_structure.path().join("docs/README.md");

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("similar")
        .arg(readme_path.to_str().unwrap())
        .arg("--limit")
        .arg("3")
        .assert()
        .success();
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

/// Test get content command
#[test]
fn test_get_command_comprehensive() {
    let test_structure = TestDirectoryStructure::new();
    let test_files = vec![
        "docs/README.md",
        "config.json",
        "data/users.csv",
        "src/main.rs",
    ];

    for file_path in test_files {
        let full_path = test_structure.path().join(file_path);
        if full_path.exists() {
            let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
            cmd.arg("get")
                .arg(full_path.to_str().unwrap())
                .assert()
                .success()
                .stdout(predicate::str::contains("content").or(predicate::str::len(10)));
        }
    }
}

/// Test get content with chunk selection
#[test]
fn test_get_with_chunks() {
    let test_structure = TestDirectoryStructure::new();
    let readme_path = test_structure.path().join("docs/README.md");

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg(readme_path.to_str().unwrap())
        .arg("--chunks")
        .arg("1-3")
        .assert()
        .success();
}

/// Test get content with single chunk
#[test]
fn test_get_single_chunk() {
    let test_structure = TestDirectoryStructure::new();
    let config_path = test_structure.path().join("config.json");

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg(config_path.to_str().unwrap())
        .arg("--chunks")
        .arg("1")
        .assert()
        .success();
}

/// Test get content with invalid chunk range
#[test]
fn test_get_invalid_chunk_range() {
    let test_structure = TestDirectoryStructure::new();
    let readme_path = test_structure.path().join("docs/README.md");

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .arg(readme_path.to_str().unwrap())
        .arg("--chunks")
        .arg("invalid-range")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("range")));
}

/// Test status command
#[test]
fn test_status_command_comprehensive() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Status")
                .or(predicate::str::contains("indexed"))
                .or(predicate::str::contains("directories"))
        );
}

/// Test status with verbose flag
#[test]
fn test_status_verbose() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::len(50)); // Should have substantial output
}

/// Test status with JSON format
#[test]
fn test_status_json_format() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{").and(predicate::str::contains("}")));
}

/// Test serve command help (should not actually start server in CI)
#[test]
fn test_serve_command_help() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("serve").or(predicate::str::contains("MCP")));
}

/// Test version command
#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

/// Test help command
#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("USAGE")
                .or(predicate::str::contains("Commands"))
                .or(predicate::str::contains("Options"))
        );
}

/// Test invalid command
#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unknown")
                .or(predicate::str::contains("invalid"))
                .or(predicate::str::contains("not found"))
        );
}

/// Test command with insufficient arguments
#[test]
fn test_insufficient_arguments_index() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));
}

#[test]
fn test_insufficient_arguments_search() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));
}

#[test]
fn test_insufficient_arguments_similar() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("similar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));
}

#[test]
fn test_insufficient_arguments_get() {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("get")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));
}

/// Test with configuration file
#[test]
fn test_with_custom_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");
    
    let config_content = r#"{
        "storage": {
            "sqlite_path": ":memory:",
            "qdrant": {
                "endpoint": "http://localhost:6333",
                "collection": "test-collection"
            }
        },
        "embedding": {
            "provider": "ollama",
            "model": "nomic-embed-text",
            "endpoint": "http://localhost:11434"
        }
    }"#;
    
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("status")
        .assert()
        .success();
}

/// End-to-end workflow test
#[test]
fn test_end_to_end_workflow() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    // Step 1: Index the directory
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Step 2: Check status
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .assert()
        .success();

    // Step 3: Search for content
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("database connection")
        .assert()
        .success();

    // Step 4: Find similar files
    let readme_path = test_structure.path().join("docs/README.md");
    if readme_path.exists() {
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("similar")
            .arg(readme_path.to_str().unwrap())
            .assert()
            .success();
    }

    // Step 5: Get content
    let config_path = test_structure.path().join("config.json");
    if config_path.exists() {
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("get")
            .arg(config_path.to_str().unwrap())
            .assert()
            .success();
    }
}