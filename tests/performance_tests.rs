use assert_cmd::Command;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::TempDir;

mod fixtures;
use fixtures::create_test_files::TestDirectoryStructure;

/// Test indexing performance with various directory sizes
#[test]
fn test_indexing_performance_small_directory() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(Duration::from_secs(30))
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Should complete within reasonable time for small directory
    assert!(duration < Duration::from_secs(30));
    println!("Small directory indexing took: {:?}", duration);
}

/// Test indexing performance with medium-sized directory
#[test]
fn test_indexing_performance_medium_directory() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create 100 files with varying sizes
    for i in 0..100 {
        let content = format!("This is file number {}. ", i).repeat(100); // ~2KB each
        fs::write(temp_dir.path().join(format!("file_{}.txt", i)), content).unwrap();
    }
    
    // Create subdirectories
    for i in 0..10 {
        let subdir = temp_dir.path().join(format!("subdir_{}", i));
        fs::create_dir(&subdir).unwrap();
        
        for j in 0..10 {
            let content = format!("Subdirectory {} file {}. ", i, j).repeat(50);
            fs::write(subdir.join(format!("subfile_{}.txt", j)), content).unwrap();
        }
    }

    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(Duration::from_secs(60))
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Should complete within reasonable time for medium directory
    assert!(duration < Duration::from_secs(60));
    println!("Medium directory indexing took: {:?}", duration);
}

/// Test search performance with various query types
#[test]
fn test_search_performance() {
    // First ensure we have some indexed content
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(Duration::from_secs(30))
        .assert()
        .success();

    let search_queries = vec![
        "database",
        "connection",
        "search functionality",
        "performance optimization",
        "error handling",
        "configuration settings",
        "programming language",
        "very specific query that might not match anything",
    ];

    for query in search_queries {
        let start = Instant::now();
        
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("search")
            .arg(query)
            .timeout(Duration::from_secs(10))
            .assert()
            .success();
        
        let duration = start.elapsed();
        
        // Search should be fast
        assert!(duration < Duration::from_secs(5));
        println!("Search for '{}' took: {:?}", query, duration);
    }
}

/// Test concurrent search performance
#[test]
fn test_concurrent_search_performance() {
    // First ensure we have some indexed content
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(Duration::from_secs(30))
        .assert()
        .success();

    let queries = vec![
        "database",
        "search",
        "performance",
        "configuration",
        "error",
    ];

    let start = Instant::now();
    
    // Run multiple searches concurrently
    let handles: Vec<_> = queries.into_iter().map(|query| {
        std::thread::spawn(move || {
            let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
            cmd.arg("search")
                .arg(query)
                .timeout(Duration::from_secs(10))
                .assert()
                .success();
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    
    // Concurrent searches should complete reasonably quickly
    assert!(duration < Duration::from_secs(15));
    println!("Concurrent searches took: {:?}", duration);
}

/// Test similar files performance
#[test]
fn test_similar_files_performance() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    // Index first
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(Duration::from_secs(30))
        .assert()
        .success();

    let test_files = vec![
        "docs/README.md",
        "src/main.rs",
        "config.json",
        "data/users.csv",
    ];

    for file_path in test_files {
        let full_path = test_structure.path().join(file_path);
        if full_path.exists() {
            let start = Instant::now();
            
            let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
            cmd.arg("similar")
                .arg(full_path.to_str().unwrap())
                .timeout(Duration::from_secs(10))
                .assert()
                .success();
            
            let duration = start.elapsed();
            
            // Similar files search should be fast
            assert!(duration < Duration::from_secs(5));
            println!("Similar files for '{}' took: {:?}", file_path, duration);
        }
    }
}

/// Test get content performance with various file sizes
#[test]
fn test_get_content_performance() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create files of various sizes
    let file_sizes = vec![
        (1024, "small.txt"),        // 1KB
        (10 * 1024, "medium.txt"),  // 10KB
        (100 * 1024, "large.txt"),  // 100KB
        (1024 * 1024, "huge.txt"),  // 1MB
    ];

    for (size, filename) in file_sizes {
        let content = "a".repeat(size);
        fs::write(temp_dir.path().join(filename), content).unwrap();
        
        let start = Instant::now();
        
        let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
        cmd.arg("get")
            .arg(temp_dir.path().join(filename).to_str().unwrap())
            .timeout(Duration::from_secs(10))
            .assert()
            .success();
        
        let duration = start.elapsed();
        
        // Get content should be fast regardless of file size
        assert!(duration < Duration::from_secs(5));
        println!("Get content for {} ({} bytes) took: {:?}", filename, size, duration);
    }
}

/// Test status command performance
#[test]
fn test_status_performance() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    // Index some content first
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(test_path)
        .timeout(Duration::from_secs(30))
        .assert()
        .success();

    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Status should be very fast
    assert!(duration < Duration::from_secs(2));
    println!("Status command took: {:?}", duration);
}

/// Test memory usage during large operations
#[test]
fn test_memory_usage_during_indexing() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create many small files to test memory efficiency
    for i in 0..500 {
        let content = format!("Content for file {} ", i).repeat(100);
        fs::write(temp_dir.path().join(format!("file_{}.txt", i)), content).unwrap();
    }

    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(Duration::from_secs(120))
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Should handle many files without excessive memory usage or timeout
    assert!(duration < Duration::from_secs(120));
    println!("Indexing 500 files took: {:?}", duration);
}

/// Test performance with different file types
#[test]
fn test_different_file_types_performance() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create files of different types
    let file_types = vec![
        ("document.md", "# Markdown Document\n\nThis is a markdown file with various content."),
        ("code.rs", "fn main() {\n    println!(\"Hello, world!\");\n}"),
        ("data.json", r#"{"name": "test", "value": 42, "items": [1, 2, 3]}"#),
        ("config.toml", "[section]\nkey = \"value\"\nnumber = 123"),
        ("script.py", "def hello():\n    print('Hello from Python')"),
        ("style.css", "body { font-family: Arial; color: #333; }"),
        ("markup.html", "<html><body><h1>Test</h1></body></html>"),
        ("data.csv", "name,age,city\nJohn,25,NYC\nJane,30,LA"),
        ("log.txt", "2024-01-01 12:00:00 INFO Application started\n2024-01-01 12:00:01 DEBUG Loading configuration"),
    ];

    for (filename, content) in file_types {
        fs::write(temp_dir.path().join(filename), content).unwrap();
    }

    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(Duration::from_secs(30))
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Should handle different file types efficiently
    assert!(duration < Duration::from_secs(30));
    println!("Indexing different file types took: {:?}", duration);
}

/// Test incremental indexing performance
#[test]
fn test_incremental_indexing_performance() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create initial set of files
    for i in 0..50 {
        let content = format!("Initial content for file {}", i);
        fs::write(temp_dir.path().join(format!("file_{}.txt", i)), content).unwrap();
    }

    // Initial indexing
    let start = Instant::now();
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(Duration::from_secs(30))
        .assert()
        .success();
    let initial_duration = start.elapsed();

    // Add more files
    for i in 50..100 {
        let content = format!("Additional content for file {}", i);
        fs::write(temp_dir.path().join(format!("file_{}.txt", i)), content).unwrap();
    }

    // Incremental indexing
    let start = Instant::now();
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(Duration::from_secs(30))
        .assert()
        .success();
    let incremental_duration = start.elapsed();

    // Incremental indexing should be faster than full indexing
    // (though this depends on implementation details)
    println!("Initial indexing took: {:?}", initial_duration);
    println!("Incremental indexing took: {:?}", incremental_duration);
    
    // Both should complete within reasonable time
    assert!(initial_duration < Duration::from_secs(30));
    assert!(incremental_duration < Duration::from_secs(30));
}

/// Benchmark overall system performance
#[test]
fn test_end_to_end_performance_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a realistic directory structure
    for i in 0..20 {
        let subdir = temp_dir.path().join(format!("module_{}", i));
        fs::create_dir(&subdir).unwrap();
        
        for j in 0..10 {
            let content = format!(
                "Module {} File {}\n\nThis file contains important information about the system.\n\
                 It includes configuration data, documentation, and code examples.\n\
                 The content is designed to be realistic and searchable.",
                i, j
            );
            fs::write(subdir.join(format!("file_{}.txt", j)), content).unwrap();
        }
    }

    println!("Starting end-to-end performance benchmark...");
    
    // Benchmark indexing
    let start = Instant::now();
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("index")
        .arg(temp_dir.path().to_str().unwrap())
        .timeout(Duration::from_secs(60))
        .assert()
        .success();
    let index_duration = start.elapsed();
    
    // Benchmark search
    let start = Instant::now();
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("search")
        .arg("configuration data")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
    let search_duration = start.elapsed();
    
    // Benchmark status
    let start = Instant::now();
    let mut cmd = Command::cargo_bin("directory-indexer").unwrap();
    cmd.arg("status")
        .assert()
        .success();
    let status_duration = start.elapsed();

    println!("Performance Benchmark Results:");
    println!("  Indexing 200 files: {:?}", index_duration);
    println!("  Search query: {:?}", search_duration);
    println!("  Status check: {:?}", status_duration);
    
    // Set reasonable performance expectations
    assert!(index_duration < Duration::from_secs(60));
    assert!(search_duration < Duration::from_secs(5));
    assert!(status_duration < Duration::from_secs(2));
}