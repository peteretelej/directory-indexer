use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[allow(dead_code)]
pub struct TestDirectoryStructure {
    pub temp_dir: TempDir,
}

impl TestDirectoryStructure {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        Self::create_comprehensive_test_structure(temp_dir.path());

        Self { temp_dir }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    #[allow(dead_code)]
    fn create_comprehensive_test_structure(base_path: &Path) {
        // Documentation files
        let docs_dir = base_path.join("docs");
        fs::create_dir_all(&docs_dir).unwrap();

        fs::write(
            docs_dir.join("README.md"),
            r#"# Project Documentation

This is the main documentation for our project. It covers:
- Installation procedures
- Configuration options
- API reference
- Troubleshooting guides

## Installation

To install this software, follow these steps:
1. Download the binary
2. Configure the settings
3. Run the initialization script

## Configuration

The configuration file should contain database settings, API keys, and performance tuning parameters.
"#,
        ).unwrap();

        fs::write(
            docs_dir.join("API.md"),
            r#"# API Reference

## Authentication

All API requests require authentication using bearer tokens.

## Endpoints

### GET /api/search
Search for documents using semantic similarity.

Parameters:
- query: string (required) - The search query
- limit: integer (optional) - Maximum results to return

### POST /api/index
Index new documents for search.

Parameters:
- path: string (required) - Directory path to index
- recursive: boolean (optional) - Whether to recurse subdirectories
"#,
        )
        .unwrap();

        fs::write(
            docs_dir.join("troubleshooting.md"),
            r#"# Troubleshooting Guide

## Common Issues

### Database Connection Errors
If you see database connection errors, check:
- Database server is running
- Network connectivity
- Authentication credentials
- Connection timeout settings

### Performance Issues
For slow query performance:
- Check database indexes
- Monitor memory usage
- Analyze query execution plans
- Consider connection pooling

### Memory Problems
If experiencing out-of-memory errors:
- Increase heap size
- Check for memory leaks
- Monitor garbage collection
- Optimize data structures
"#,
        )
        .unwrap();

        // Source code files
        let src_dir = base_path.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            src_dir.join("main.rs"),
            r#"use std::env;
use std::process;

mod config;
mod database;
mod search;
mod indexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        process::exit(1);
    }

    match args[1].as_str() {
        "index" => {
            if args.len() < 3 {
                eprintln!("Usage: {} index <directory>", args[0]);
                process::exit(1);
            }
            indexer::index_directory(&args[2]);
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: {} search <query>", args[0]);
                process::exit(1);
            }
            search::search_documents(&args[2]);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            process::exit(1);
        }
    }
}
"#,
        )
        .unwrap();

        fs::write(
            src_dir.join("database.rs"),
            r#"use std::collections::HashMap;
use std::error::Error;

pub struct Database {
    connection_string: String,
    pool: Option<ConnectionPool>,
}

pub struct ConnectionPool {
    max_connections: usize,
    active_connections: usize,
}

impl Database {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            pool: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        // Initialize connection pool
        self.pool = Some(ConnectionPool {
            max_connections: 10,
            active_connections: 0,
        });
        Ok(())
    }

    pub fn execute_query(&self, sql: &str) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
        // Execute SQL query and return results
        let mut results = Vec::new();
        
        // Simulate query execution
        if sql.contains("SELECT") {
            let mut row = HashMap::new();
            row.insert("id".to_string(), "1".to_string());
            row.insert("content".to_string(), "sample data".to_string());
            results.push(row);
        }
        
        Ok(results)
    }
}
"#,
        )
        .unwrap();

        fs::write(
            src_dir.join("search.rs"),
            r#"use crate::database::Database;
use std::collections::HashMap;

pub struct SearchEngine {
    database: Database,
    index: HashMap<String, Vec<usize>>,
}

impl SearchEngine {
    pub fn new(db: Database) -> Self {
        Self {
            database: db,
            index: HashMap::new(),
        }
    }

    pub fn index_document(&mut self, doc_id: usize, content: &str) {
        let words: Vec<&str> = content.split_whitespace().collect();
        
        for word in words {
            let word_lower = word.to_lowercase();
            self.index.entry(word_lower)
                .or_insert_with(Vec::new)
                .push(doc_id);
        }
    }

    pub fn search(&self, query: &str) -> Vec<usize> {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let mut results = Vec::new();
        
        for word in query_words {
            if let Some(doc_ids) = self.index.get(&word.to_lowercase()) {
                results.extend(doc_ids);
            }
        }
        
        results.sort();
        results.dedup();
        results
    }
}

pub fn search_documents(query: &str) {
    println!("Searching for: {}", query);
    // Implementation would use SearchEngine
}
"#,
        )
        .unwrap();

        // Configuration files
        fs::write(
            base_path.join("config.json"),
            r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "search_db",
    "user": "app_user",
    "connection_pool": {
      "max_connections": 20,
      "idle_timeout": 300,
      "connection_timeout": 10
    }
  },
  "search": {
    "index_batch_size": 1000,
    "similarity_threshold": 0.7,
    "max_results": 100
  },
  "logging": {
    "level": "info",
    "format": "json",
    "output": "stdout"
  },
  "performance": {
    "cache_size": "512MB",
    "worker_threads": 4,
    "enable_compression": true
  }
}
"#,
        )
        .unwrap();

        fs::write(
            base_path.join("Cargo.toml"),
            r#"[package]
name = "search-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3.0"
assert_cmd = "2.0"
predicates = "3.0"
"#,
        )
        .unwrap();

        fs::write(
            base_path.join("package.json"),
            r#"{
  "name": "search-frontend",
  "version": "1.0.0",
  "description": "Frontend for search engine",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "jest",
    "build": "webpack --mode=production",
    "dev": "webpack-dev-server --mode=development"
  },
  "dependencies": {
    "express": "^4.18.2",
    "axios": "^1.4.0",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "jest": "^29.5.0",
    "webpack": "^5.88.0",
    "webpack-cli": "^5.1.0"
  },
  "keywords": ["search", "api", "frontend"],
  "author": "Development Team",
  "license": "MIT"
}
"#,
        )
        .unwrap();

        // Data files
        let data_dir = base_path.join("data");
        fs::create_dir_all(&data_dir).unwrap();

        fs::write(
            data_dir.join("users.csv"),
            r#"id,name,email,role,created_at,last_login
1,John Doe,john@example.com,admin,2023-01-15,2024-01-20
2,Jane Smith,jane@example.com,user,2023-02-20,2024-01-19
3,Bob Johnson,bob@example.com,user,2023-03-10,2024-01-18
4,Alice Brown,alice@example.com,moderator,2023-04-05,2024-01-17
5,Charlie Wilson,charlie@example.com,user,2023-05-12,2024-01-16
"#,
        )
        .unwrap();

        fs::write(
            data_dir.join("products.json"),
            r#"{
  "products": [
    {
      "id": 1,
      "name": "Laptop Computer",
      "description": "High-performance laptop with 16GB RAM and SSD storage",
      "price": 1299.99,
      "category": "electronics",
      "tags": ["computer", "laptop", "portable", "work"]
    },
    {
      "id": 2,
      "name": "Office Chair",
      "description": "Ergonomic office chair with lumbar support and adjustable height",
      "price": 249.99,
      "category": "furniture",
      "tags": ["chair", "office", "ergonomic", "comfort"]
    },
    {
      "id": 3,
      "name": "Coffee Maker",
      "description": "Programmable coffee maker with thermal carafe and auto-shutoff",
      "price": 89.99,
      "category": "appliances",
      "tags": ["coffee", "kitchen", "programmable", "thermal"]
    }
  ]
}
"#,
        )
        .unwrap();

        fs::write(
            data_dir.join("logs.txt"),
            r#"2024-01-20 10:30:15 INFO Starting application server
2024-01-20 10:30:16 INFO Database connection established
2024-01-20 10:30:16 INFO Search index loaded successfully
2024-01-20 10:35:22 DEBUG User search query: "laptop performance"
2024-01-20 10:35:22 DEBUG Found 15 matching documents
2024-01-20 10:35:23 INFO Search completed in 0.245s
2024-01-20 10:42:18 WARN Connection timeout for user session abc123
2024-01-20 10:42:19 INFO User session abc123 reconnected
2024-01-20 11:15:33 ERROR Failed to process file: permission denied
2024-01-20 11:15:34 INFO Retrying file processing with elevated privileges
2024-01-20 11:15:35 INFO File processing completed successfully
"#,
        )
        .unwrap();

        // Scripts and utilities
        let scripts_dir = base_path.join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();

        fs::write(
            scripts_dir.join("setup.sh"),
            r#"#!/bin/bash

echo "Setting up development environment..."

# Install dependencies
cargo build
npm install

# Set up database
createdb search_db
psql search_db < schema.sql

# Initialize search index
cargo run -- index ./sample_data

echo "Setup complete!"
"#,
        )
        .unwrap();

        fs::write(
            scripts_dir.join("deploy.py"),
            r#"#!/usr/bin/env python3

import os
import subprocess
import sys

def run_command(cmd):
    """Run a shell command and handle errors."""
    print(f"Running: {cmd}")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)
    
    return result.stdout

def main():
    print("Starting deployment process...")
    
    # Build application
    run_command("cargo build --release")
    
    # Run tests
    run_command("cargo test")
    
    # Deploy to server
    run_command("scp target/release/search-engine user@server:/opt/app/")
    
    # Restart service
    run_command("ssh user@server 'systemctl restart search-engine'")
    
    print("Deployment completed successfully!")

if __name__ == "__main__":
    main()
"#,
        )
        .unwrap();

        // Test files
        let tests_dir = base_path.join("tests");
        fs::create_dir_all(&tests_dir).unwrap();

        fs::write(
            tests_dir.join("unit_tests.rs"),
            r#"#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_connection() {
        let mut db = Database::new("test://localhost");
        assert!(db.connect().is_ok());
    }

    #[test]  
    fn test_search_functionality() {
        let db = Database::new("test://localhost");
        let mut engine = SearchEngine::new(db);
        
        engine.index_document(1, "rust programming language");
        engine.index_document(2, "python programming tutorial");
        
        let results = engine.search("programming");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_empty_search() {
        let db = Database::new("test://localhost");
        let engine = SearchEngine::new(db);
        
        let results = engine.search("nonexistent");
        assert!(results.is_empty());
    }
}
"#,
        )
        .unwrap();

        // Nested directories with specialized content
        let backend_dir = base_path.join("backend/api");
        fs::create_dir_all(&backend_dir).unwrap();

        fs::write(
            backend_dir.join("handlers.go"),
            r#"package api

import (
    "encoding/json"
    "net/http"
    "strconv"
)

type SearchRequest struct {
    Query string `json:"query"`
    Limit int    `json:"limit"`
}

type SearchResponse struct {
    Results []Document `json:"results"`
    Total   int        `json:"total"`
}

type Document struct {
    ID      int     `json:"id"`
    Title   string  `json:"title"`
    Content string  `json:"content"`
    Score   float64 `json:"score"`
}

func SearchHandler(w http.ResponseWriter, r *http.Request) {
    if r.Method != http.MethodPost {
        http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
        return
    }

    var req SearchRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, "Invalid JSON", http.StatusBadRequest)
        return
    }

    // Simulate search operation
    documents := []Document{
        {ID: 1, Title: "Sample Document", Content: "This is sample content", Score: 0.95},
        {ID: 2, Title: "Another Document", Content: "More sample content", Score: 0.87},
    }

    response := SearchResponse{
        Results: documents,
        Total:   len(documents),
    }

    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(response)
}
"#,
        )
        .unwrap();

        let frontend_dir = base_path.join("frontend/components");
        fs::create_dir_all(&frontend_dir).unwrap();

        fs::write(
            frontend_dir.join("SearchBox.tsx"),
            r#"import React, { useState, useCallback } from 'react';

interface SearchBoxProps {
  onSearch: (query: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

export const SearchBox: React.FC<SearchBoxProps> = ({
  onSearch,
  placeholder = "Enter search query...",
  disabled = false
}) => {
  const [query, setQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!query.trim() || disabled) return;

    setIsLoading(true);
    try {
      await onSearch(query.trim());
    } finally {
      setIsLoading(false);
    }
  }, [query, onSearch, disabled]);

  return (
    <form onSubmit={handleSubmit} className="search-box">
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={placeholder}
        disabled={disabled || isLoading}
        className="search-input"
      />
      <button
        type="submit"
        disabled={disabled || isLoading || !query.trim()}
        className="search-button"
      >
        {isLoading ? 'Searching...' : 'Search'}
      </button>
    </form>
  );
};
"#,
        )
        .unwrap();

        // Error logs and edge cases
        let logs_dir = base_path.join("logs");
        fs::create_dir_all(&logs_dir).unwrap();

        fs::write(
            logs_dir.join("error.log"),
            r#"[2024-01-20T10:30:15Z] ERROR: Database connection failed: connection timeout
[2024-01-20T10:30:16Z] WARN: Retrying database connection (attempt 1/3)
[2024-01-20T10:30:18Z] INFO: Database connection established
[2024-01-20T10:35:22Z] ERROR: Search query failed: invalid syntax in query "test AND OR"
[2024-01-20T10:35:23Z] DEBUG: Falling back to simple text search
[2024-01-20T10:42:18Z] ERROR: File indexing failed: permission denied for /protected/file.txt
[2024-01-20T10:42:19Z] WARN: Skipping protected file due to access restrictions
[2024-01-20T11:15:33Z] ERROR: Out of memory during large file processing
[2024-01-20T11:15:34Z] INFO: Implementing chunked processing for large files
[2024-01-20T11:15:35Z] INFO: Large file processing completed successfully
"#,
        )
        .unwrap();

        // Binary-like files (should be ignored)
        fs::write(
            base_path.join("binary.dat"),
            [
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            ],
        )
        .unwrap();

        // Large text file for performance testing
        let mut large_content = String::new();
        for i in 0..50 {
            // Reduced from 1000 to 50 for faster testing
            large_content.push_str(&format!(
                "Line {} - This is a large file used for performance testing. It contains repeated content to simulate real-world large documents.\n",
                i
            ));
        }
        fs::write(base_path.join("large_file.txt"), large_content).unwrap();
    }
}
