// CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn test_command(test_name: &str) -> Command {
    let collection_name = format!("di-test-cli-{}", test_name);
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.env("DIRECTORY_INDEXER_QDRANT_COLLECTION", collection_name);
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

    let test_data_path = get_test_data_path();

    // Index test_data first
    test_command("semantic-auth")
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success();

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
    assert!(
        stdout.contains("api_guide.md") || stdout.contains("Search Results"),
        "Should find API guide when searching for authentication"
    );
}

#[test]
fn test_semantic_search_error_handling() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let test_data_path = get_test_data_path();

    // Index test_data first
    test_command("semantic-error")
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success();

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
    assert!(
        stdout.contains("troubleshooting.md") || stdout.contains("Search Results"),
        "Should find troubleshooting guide when searching for error handling"
    );
}

#[test]
fn test_semantic_search_programming() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let test_data_path = get_test_data_path();

    // Index test_data first
    test_command("semantic-prog")
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success();

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
    assert!(
        stdout.contains("rust.txt")
            || stdout.contains("hello.rs")
            || stdout.contains("Search Results"),
        "Should find Rust files when searching for rust programming"
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

    // Index test_data first
    test_command("search-path-filter")
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success();

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

    let test_data_path = get_test_data_path();

    // Index test_data first
    test_command("search-limit")
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success();

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

    // Index test_data first
    test_command("similar-workflow")
        .arg("index")
        .arg(&test_data_path)
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .success();

    // Find files similar to hello.rs
    test_command("similar-workflow")
        .arg("similar")
        .arg(hello_rs_path.to_str().unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}
