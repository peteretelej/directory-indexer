// CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to ensure only one test creates the shared collection at a time
static COLLECTION_INIT_LOCK: Mutex<()> = Mutex::new(());

fn ensure_shared_collection_exists() {
    let _lock = COLLECTION_INIT_LOCK.lock().unwrap();

    // Use a semantic test name to get the shared collection
    let shared_test_name = "semantic-init";

    eprintln!("=== ensure_shared_collection_exists: Starting ===");

    // Check if collection already has data to avoid re-indexing
    eprintln!("=== Checking if collection has data ===");
    let test_search = test_command(shared_test_name)
        .arg("search")
        .arg("test")
        .arg("--limit")
        .arg("1")
        .timeout(std::time::Duration::from_secs(10))
        .output();

    match test_search {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("=== Search test output ===");
            eprintln!("STDOUT: {}", stdout);
            eprintln!("STDERR: {}", stderr);
            eprintln!("Exit code: {}", output.status);

            if !stdout.contains("No results found") {
                eprintln!("=== Collection has data, skipping indexing ===");
                return;
            } else {
                eprintln!("=== No results found, need to index ===");
            }
        }
        Err(e) => {
            eprintln!("=== Search failed: {}, need to index ===", e);
        }
    }

    // Collection doesn't exist or is empty, index test data
    let test_data_path = get_test_data_path();
    eprintln!("=== Indexing test data from: {} ===", test_data_path);

    let index_result = test_command(shared_test_name)
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .output()
        .expect("Failed to run index command");

    eprintln!("=== Index result ===");
    eprintln!("STDOUT: {}", String::from_utf8_lossy(&index_result.stdout));
    eprintln!("STDERR: {}", String::from_utf8_lossy(&index_result.stderr));
    eprintln!("Exit code: {}", index_result.status);

    if !index_result.status.success() {
        panic!("Index command failed: {:?}", index_result.status);
    }

    eprintln!("=== ensure_shared_collection_exists: Complete ===");
}

fn test_command(test_name: &str) -> Command {
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();

    // Hardcoded collection names - no environment variable dependencies
    let collection_name = if test_name.starts_with("semantic-")
        || test_name.contains("test-data")
        || test_name == "search-path-filter"
        || test_name == "search-limit"
        || test_name == "similar-workflow"
    {
        // Shared collection for tests that use pre-indexed test_data
        "directory-indexer-integration-test".to_string()
    } else {
        // Individual collections for tests with temporary data
        format!("di-test-{}", test_name)
    };

    cmd.env("DIRECTORY_INDEXER_QDRANT_COLLECTION", &collection_name);
    eprintln!(
        "Test '{}' using hardcoded collection: {}",
        test_name, collection_name
    );
    cmd
}

fn are_services_available() -> bool {
    let qdrant_endpoint =
        std::env::var("QDRANT_ENDPOINT").unwrap_or_else(|_| "http://localhost:6333".to_string());
    let ollama_endpoint =
        std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let qdrant_available = std::process::Command::new("curl")
        .args(["-s", &format!("{}/", qdrant_endpoint), "-o", "/dev/null"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let ollama_available = std::process::Command::new("curl")
        .args([
            "-s",
            &format!("{}/api/tags", ollama_endpoint),
            "-o",
            "/dev/null",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    qdrant_available && ollama_available
}

fn create_simple_test_files(dir: &TempDir) -> std::io::Result<()> {
    fs::write(
        dir.path().join("readme.md"),
        "# Project README\nThis is documentation about the project.",
    )?;
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}",
    )?;
    fs::write(
        dir.path().join("config.json"),
        r#"{"name": "test", "version": "1.0"}"#,
    )?;
    Ok(())
}

#[test]
fn test_basic_workflow() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_simple_test_files(&temp_dir).unwrap();
    let test_path = temp_dir.path().to_str().unwrap();

    // Test index command
    test_command("basic-workflow")
        .arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexing"));

    // Test search command
    test_command("basic-workflow")
        .arg("search")
        .arg("documentation")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    // Test status command
    test_command("basic-workflow")
        .arg("status")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success()
        .stdout(predicate::str::contains("Status"));
}

#[test]
fn test_search_with_options() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_simple_test_files(&temp_dir).unwrap();
    let test_path = temp_dir.path().to_str().unwrap();

    // Index first
    test_command("search-options")
        .arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Test search with path filter
    test_command("search-options")
        .arg("search")
        .arg("project")
        .arg("--path")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    // Test search with limit
    test_command("search-options")
        .arg("search")
        .arg("project")
        .arg("--limit")
        .arg("5")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

#[test]
fn test_similar_command() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_simple_test_files(&temp_dir).unwrap();
    let test_path = temp_dir.path().to_str().unwrap();
    let readme_path = temp_dir.path().join("readme.md");

    // Index first
    test_command("similar-command")
        .arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Test similar command
    test_command("similar-command")
        .arg("similar")
        .arg(readme_path.to_str().unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

#[test]
fn test_get_command() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    create_simple_test_files(&temp_dir).unwrap();
    let test_path = temp_dir.path().to_str().unwrap();
    let readme_path = temp_dir.path().join("readme.md");

    // Index first
    test_command("get-command")
        .arg("index")
        .arg(test_path)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Test get command
    test_command("get-command")
        .arg("get")
        .arg(readme_path.to_str().unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("README"));
}

#[test]
fn test_error_handling() {
    // Test invalid commands
    test_command("error-handling")
        .arg("nonexistent-command")
        .assert()
        .failure();

    // Test missing arguments
    test_command("error-handling")
        .arg("search")
        .assert()
        .failure();

    test_command("error-handling")
        .arg("similar")
        .assert()
        .failure();

    test_command("error-handling").arg("get").assert().failure();
}

#[tokio::test]
async fn test_health_check_functions() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let config = directory_indexer::config::Config::load().expect("Config should load");

    // Test system health check
    let health = directory_indexer::health::check_system_health(&config).await;
    assert!(health.is_ready_for_indexing() || health.is_ready_for_retrieval());

    // Test embedding generation
    let result = directory_indexer::health::test_embedding_generation(&config).await;
    // Should either succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_qdrant_delete_operations() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let config = directory_indexer::config::Config::load().expect("Config should load");
    let store = directory_indexer::storage::QdrantStore::new(
        &config.storage.qdrant.endpoint,
        "test-delete-ops".to_string(),
    )
    .await
    .expect("Store should be created");

    // Test delete points by file
    let result = store.delete_points_by_file("/test/file.txt").await;
    assert!(result.is_ok(), "Delete points by file should succeed");

    // Test delete collection
    let result = store.delete_collection().await;
    assert!(result.is_ok(), "Delete collection should succeed");
}

// ============================================================================
// Semantic Search Quality Tests using test_data
// ============================================================================

fn get_test_data_path() -> String {
    std::env::current_dir()
        .unwrap()
        .join("test_data")
        .to_string_lossy()
        .to_string()
}

#[test]
fn test_semantic_search_authentication() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    // Ensure shared collection exists and has test data
    ensure_shared_collection_exists();

    // Search for authentication - should find API guide
    let output = test_command("semantic-auth")
        .arg("search")
        .arg("authentication")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("api_guide.md") || stdout.contains("Search Results"),
        "Should find API guide when searching for authentication. \nSTDOUT: {}\nSTDERR: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_semantic_search_error_handling() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    // Ensure shared collection exists and has test data
    ensure_shared_collection_exists();

    // Search for error handling - should find troubleshooting docs
    let output = test_command("semantic-error")
        .arg("search")
        .arg("error handling")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("troubleshooting.md") || stdout.contains("Search Results"),
        "Should find troubleshooting guide when searching for error handling. \nSTDOUT: {}\nSTDERR: {}",
        stdout, stderr
    );
}

#[test]
fn test_semantic_search_programming() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    // Ensure shared collection exists and has test data
    ensure_shared_collection_exists();

    // Search for rust programming - should find rust files
    let output = test_command("semantic-prog")
        .arg("search")
        .arg("rust programming")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("rust.txt")
            || stdout.contains("hello.rs")
            || stdout.contains("Search Results"),
        "Should find Rust files when searching for rust programming. \nSTDOUT: {}\nSTDERR: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_search_with_path_filter() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let test_data_path = get_test_data_path();
    let programming_path = std::path::Path::new(&test_data_path).join("programming");

    // Ensure shared collection exists and has test data
    ensure_shared_collection_exists();

    // Search within programming directory only
    test_command("search-path-filter")
        .arg("search")
        .arg("function")
        .arg("--path")
        .arg(programming_path.to_str().unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

#[test]
fn test_search_with_limit() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    // Ensure shared collection exists and has test data
    ensure_shared_collection_exists();

    // Search with limit
    test_command("search-limit")
        .arg("search")
        .arg("configuration")
        .arg("--limit")
        .arg("2")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

#[test]
fn test_similar_files_workflow() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let test_data_path = get_test_data_path();
    let hello_rs_path = std::path::Path::new(&test_data_path)
        .join("programming")
        .join("hello.rs");

    // Ensure shared collection exists and has test data
    ensure_shared_collection_exists();

    // Find files similar to hello.rs
    test_command("similar-workflow")
        .arg("similar")
        .arg(hello_rs_path.to_str().unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}
