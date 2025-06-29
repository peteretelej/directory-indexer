use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

mod fixtures;
use fixtures::create_test_files::TestDirectoryStructure;

mod common;
use common::test_env::TestEnvironment;

/// Helper struct to manage MCP server process
struct McpServerHandle {
    process: std::process::Child,
}

impl McpServerHandle {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let process = Command::new("cargo")
            .args(["run", "--", "serve"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Give the server a moment to start
        thread::sleep(Duration::from_millis(1000));

        Ok(Self { process })
    }

    fn send_request(&mut self, request: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let stdin = self.process.stdin.as_mut().unwrap();
        let request_str = serde_json::to_string(&request)?;

        stdin.write_all(request_str.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;

        // Read response
        let stdout = self.process.stdout.as_mut().unwrap();
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;

        let response: Value = serde_json::from_str(&response_line)?;
        Ok(response)
    }
}

impl Drop for McpServerHandle {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Test MCP server initialization
#[test]
fn test_mcp_server_initialization() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": true
                },
                "sampling": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["capabilities"].is_object());
}

/// Test server_info tool
#[test]
fn test_server_info_tool() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "server_info",
            "arguments": {}
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());

    let content = &response["result"]["content"][0];
    assert_eq!(content["type"], "text");
    assert!(content["text"]
        .as_str()
        .unwrap()
        .contains("Directory Indexer"));
}

/// Test index tool
#[tokio::test]
async fn test_index_tool() {
    let _env = TestEnvironment::new("mcp-index-tool").await;
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_path
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());

    let content = &response["result"]["content"][0];
    assert_eq!(content["type"], "text");
    assert!(
        content["text"].as_str().unwrap().contains("indexed")
            || content["text"].as_str().unwrap().contains("Indexing")
    );
}

/// Test index tool with multiple directories
#[tokio::test]
async fn test_index_tool_multiple_directories() {
    let _env = TestEnvironment::new("mcp-index-tool-multiple").await;
    let test_structure1 = TestDirectoryStructure::new();
    let test_structure2 = TestDirectoryStructure::new();

    let path1 = test_structure1.path().to_str().unwrap();
    let path2 = test_structure2.path().to_str().unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": format!("{},{}", path1, path2)
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

/// Test index tool with invalid directory
#[test]
fn test_index_tool_invalid_directory() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": "/path/that/does/not/exist"
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["error"].is_object()
            || response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("error")
    );
}

/// Test search tool
#[test]
fn test_search_tool() {
    let test_queries = vec![
        "database connection",
        "search functionality",
        "configuration settings",
        "error handling",
        "performance optimization",
    ];

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    for query in test_queries {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "search",
                "arguments": {
                    "query": query
                }
            }
        });

        let response = server
            .send_request(request)
            .expect("Failed to send request");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"]["content"].is_array());

        let content = &response["result"]["content"][0];
        assert_eq!(content["type"], "text");
        assert!(!content["text"].as_str().unwrap().is_empty());
    }
}

/// Test search tool with directory scope
#[test]
fn test_search_tool_with_directory_scope() {
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": "database",
                "directory_path": test_path
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

/// Test search tool with limit
#[test]
fn test_search_tool_with_limit() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": "test",
                "limit": 3
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

/// Test search tool with empty query
#[test]
fn test_search_tool_empty_query() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": ""
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["error"].is_object()
            || response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("empty")
    );
}

/// Test similar_files tool
#[test]
fn test_similar_files_tool() {
    let test_structure = TestDirectoryStructure::new();
    let test_files = vec![
        "docs/README.md",
        "src/main.rs",
        "config.json",
        "data/users.csv",
    ];

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    for file_path in test_files {
        let full_path = test_structure.path().join(file_path);
        if full_path.exists() {
            let request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "similar_files",
                    "arguments": {
                        "file_path": full_path.to_str().unwrap()
                    }
                }
            });

            let response = server
                .send_request(request)
                .expect("Failed to send request");

            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["id"], 1);
            assert!(response["result"]["content"].is_array());

            let content = &response["result"]["content"][0];
            assert_eq!(content["type"], "text");
            assert!(!content["text"].as_str().unwrap().is_empty());
        }
    }
}

/// Test similar_files tool with limit
#[test]
fn test_similar_files_tool_with_limit() {
    let test_structure = TestDirectoryStructure::new();
    let readme_path = test_structure.path().join("docs/README.md");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "similar_files",
            "arguments": {
                "file_path": readme_path.to_str().unwrap(),
                "limit": 5
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

/// Test similar_files tool with non-existent file
#[test]
fn test_similar_files_tool_nonexistent_file() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "similar_files",
            "arguments": {
                "file_path": "/path/to/nonexistent/file.txt"
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["error"].is_object()
            || response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("not found")
    );
}

/// Test get_content tool
#[test]
fn test_get_content_tool() {
    let test_structure = TestDirectoryStructure::new();
    let test_files = vec![
        "docs/README.md",
        "config.json",
        "data/users.csv",
        "src/main.rs",
    ];

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    for file_path in test_files {
        let full_path = test_structure.path().join(file_path);
        if full_path.exists() {
            let request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get_content",
                    "arguments": {
                        "file_path": full_path.to_str().unwrap()
                    }
                }
            });

            let response = server
                .send_request(request)
                .expect("Failed to send request");

            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["id"], 1);
            assert!(response["result"]["content"].is_array());

            let content = &response["result"]["content"][0];
            assert_eq!(content["type"], "text");
            assert!(!content["text"].as_str().unwrap().is_empty());
        }
    }
}

/// Test get_content tool with chunk selection
#[test]
fn test_get_content_tool_with_chunks() {
    let test_structure = TestDirectoryStructure::new();
    let readme_path = test_structure.path().join("docs/README.md");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "get_content",
            "arguments": {
                "file_path": readme_path.to_str().unwrap(),
                "chunks": "1-3"
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

/// Test get_content tool with single chunk
#[test]
fn test_get_content_tool_single_chunk() {
    let test_structure = TestDirectoryStructure::new();
    let config_path = test_structure.path().join("config.json");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "get_content",
            "arguments": {
                "file_path": config_path.to_str().unwrap(),
                "chunks": "1"
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

/// Test get_content tool with invalid chunk range
#[test]
fn test_get_content_tool_invalid_chunk_range() {
    let test_structure = TestDirectoryStructure::new();
    let readme_path = test_structure.path().join("docs/README.md");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "get_content",
            "arguments": {
                "file_path": readme_path.to_str().unwrap(),
                "chunks": "invalid-range"
            }
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["error"].is_object()
            || response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("invalid")
    );
}

/// Test listing available tools
#[test]
fn test_list_tools() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert!(tool_names.contains(&"index".to_string()));
    assert!(tool_names.contains(&"search".to_string()));
    assert!(tool_names.contains(&"similar_files".to_string()));
    assert!(tool_names.contains(&"get_content".to_string()));
    assert!(tool_names.contains(&"server_info".to_string()));
}

/// Test tool with missing required arguments
#[test]
fn test_tool_missing_arguments() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {}
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("required")
            || response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("missing")
    );
}

/// Test calling non-existent tool
#[test]
fn test_nonexistent_tool() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let response = server
        .send_request(request)
        .expect("Failed to send request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not found")
            || response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("unknown")
    );
}

/// Test end-to-end MCP workflow
#[tokio::test]
async fn test_mcp_end_to_end_workflow() {
    let _env = TestEnvironment::new("mcp-end-to-end-workflow").await;
    let test_structure = TestDirectoryStructure::new();
    let test_path = test_structure.path().to_str().unwrap();

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    // Step 1: Initialize server
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response = server
        .send_request(init_request)
        .expect("Failed to initialize");
    assert_eq!(response["jsonrpc"], "2.0");

    // Step 2: Get server info
    let info_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "server_info",
            "arguments": {}
        }
    });

    let response = server
        .send_request(info_request)
        .expect("Failed to get server info");
    assert_eq!(response["jsonrpc"], "2.0");

    // Step 3: Index directory
    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_path
            }
        }
    });

    let response = server.send_request(index_request).expect("Failed to index");
    assert_eq!(response["jsonrpc"], "2.0");

    // Step 4: Search for content
    let search_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": "database connection"
            }
        }
    });

    let response = server
        .send_request(search_request)
        .expect("Failed to search");
    assert_eq!(response["jsonrpc"], "2.0");

    // Step 5: Find similar files
    let readme_path = test_structure.path().join("docs/README.md");
    if readme_path.exists() {
        let similar_request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "similar_files",
                "arguments": {
                    "file_path": readme_path.to_str().unwrap()
                }
            }
        });

        let response = server
            .send_request(similar_request)
            .expect("Failed to find similar files");
        assert_eq!(response["jsonrpc"], "2.0");
    }

    // Step 6: Get file content
    let config_path = test_structure.path().join("config.json");
    if config_path.exists() {
        let content_request = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "get_content",
                "arguments": {
                    "file_path": config_path.to_str().unwrap()
                }
            }
        });

        let response = server
            .send_request(content_request)
            .expect("Failed to get content");
        assert_eq!(response["jsonrpc"], "2.0");
    }
}
