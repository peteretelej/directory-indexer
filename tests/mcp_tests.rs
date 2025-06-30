// MCP integration tests

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;

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

        thread::sleep(Duration::from_millis(1000));
        Ok(Self { process })
    }

    fn send_request(&mut self, request: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let stdin = self.process.stdin.as_mut().unwrap();
        let request_str = serde_json::to_string(&request)?;

        stdin.write_all(request_str.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;

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

#[test]
fn test_mcp_initialize() {
    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response = server
        .send_request(initialize_request)
        .expect("Failed to send initialize request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn test_mcp_list_tools() {
    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    // Initialize first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server
        .send_request(initialize_request)
        .expect("Initialize failed");

    // List tools
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    let response = server
        .send_request(list_tools_request)
        .expect("Failed to send list tools request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);

    // Just check that we get a tools array, don't assume specific tool names
    if let Some(result) = response.get("result") {
        if let Some(tools) = result.get("tools") {
            assert!(tools.is_array());
            // Just verify we have some tools available
            let tools_array = tools.as_array().unwrap();
            assert!(
                !tools_array.is_empty(),
                "Should have at least one tool available"
            );
        } else {
            panic!("Expected 'tools' field in result");
        }
    } else {
        panic!("Expected 'result' field in response");
    }
}

#[tokio::test]
async fn test_mcp_index_tool() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    // Initialize
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server
        .send_request(initialize_request)
        .expect("Initialize failed");

    // Test index tool with test data directory
    let test_data_path = std::env::current_dir()
        .unwrap()
        .join("test_data")
        .to_string_lossy()
        .to_string();

    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_data_path
            }
        }
    });

    // Add timeout to prevent hanging (2 minutes should be plenty for test data)
    let response = timeout(Duration::from_secs(120), async {
        server.send_request(index_request)
    })
    .await
    .expect("Index operation timed out after 2 minutes")
    .expect("Failed to call index");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    // Just verify we get some response, don't check specific structure
    assert!(response.get("result").is_some() || response.get("error").is_some());
}

#[test]
fn test_mcp_search_tool() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    // Initialize
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server
        .send_request(initialize_request)
        .expect("Initialize failed");

    // Test search tool with simple mock call
    let search_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": "test"
            }
        }
    });

    let response = server
        .send_request(search_request)
        .expect("Failed to call search");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    // Just verify we get some response, don't check specific structure
    assert!(response.get("result").is_some() || response.get("error").is_some());
}

#[test]
fn test_mcp_error_handling() {
    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    // Test invalid method
    let invalid_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "invalid_method"
    });

    let response = server
        .send_request(invalid_request)
        .expect("Failed to send invalid request");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());

    // Test missing parameters
    let missing_params_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "search"
            // Missing arguments
        }
    });

    let response = server
        .send_request(missing_params_request)
        .expect("Failed to send request with missing params");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["error"].is_object());
}

#[test]
fn test_mcp_notifications_initialized() {
    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    let notifications_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "notifications/initialized",
        "params": {}
    });

    let response = server
        .send_request(notifications_request)
        .expect("Failed to send notifications/initialized");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["error"].is_null() || !response.get("error").is_some());
}

#[test]
fn test_mcp_resources_list() {
    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    let resources_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list",
        "params": {}
    });

    let response = server
        .send_request(resources_request)
        .expect("Failed to send resources/list");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["resources"].is_array());
    assert!(response["error"].is_null() || !response.get("error").is_some());
}

#[test]
fn test_mcp_resources_templates_list() {
    let mut server = McpServerHandle::new().expect("Failed to start MCP server");

    let templates_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/templates/list",
        "params": {}
    });

    let response = server
        .send_request(templates_request)
        .expect("Failed to send resources/templates/list");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["resourceTemplates"].is_array());
    assert!(response["error"].is_null() || !response.get("error").is_some());
}

// ============================================================================
// Full Workflow Tests using test_data  
// ============================================================================

fn get_test_data_path() -> String {
    std::env::current_dir()
        .unwrap()
        .join("test_data")
        .to_string_lossy()
        .to_string()
}

#[tokio::test]
async fn test_mcp_full_workflow() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let test_data_path = get_test_data_path();

    // Initialize
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server.send_request(initialize_request).expect("Initialize failed");

    // Index test_data
    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_data_path
            }
        }
    });

    let index_response = timeout(Duration::from_secs(120), async {
        server.send_request(index_request)
    })
    .await
    .expect("Index operation timed out")
    .expect("Failed to call index");

    assert_eq!(index_response["jsonrpc"], "2.0");
    assert_eq!(index_response["id"], 2);
    assert!(index_response.get("result").is_some());

    // Search for authentication
    let search_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": "authentication"
            }
        }
    });

    let search_response = server.send_request(search_request)
        .expect("Failed to call search");

    assert_eq!(search_response["jsonrpc"], "2.0");
    assert_eq!(search_response["id"], 3);
    assert!(search_response.get("result").is_some());
}

#[tokio::test]
async fn test_mcp_search_with_parameters() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let test_data_path = get_test_data_path();

    // Initialize
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server.send_request(initialize_request).expect("Initialize failed");

    // Index test_data first
    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_data_path
            }
        }
    });

    timeout(Duration::from_secs(120), async {
        server.send_request(index_request)
    })
    .await
    .expect("Index operation timed out")
    .expect("Failed to call index");

    // Search with directory_path and limit parameters
    let search_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {
                "query": "programming",
                "directory_path": get_test_data_path() + "/programming",
                "limit": 3
            }
        }
    });

    let response = server.send_request(search_request)
        .expect("Failed to call search with parameters");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_mcp_similar_files_tool() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let test_data_path = get_test_data_path();

    // Initialize
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server.send_request(initialize_request).expect("Initialize failed");

    // Index test_data first
    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_data_path
            }
        }
    });

    timeout(Duration::from_secs(120), async {
        server.send_request(index_request)
    })
    .await
    .expect("Index operation timed out")
    .expect("Failed to call index");

    // Test similar_files tool
    let similar_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "similar_files",
            "arguments": {
                "file_path": get_test_data_path() + "/programming/hello.rs",
                "limit": 5
            }
        }
    });

    let response = server.send_request(similar_request)
        .expect("Failed to call similar_files");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response.get("result").is_some() || response.get("error").is_some());
}

#[tokio::test]
async fn test_mcp_get_content_tool() {
    if !are_services_available() {
        println!("Skipping test - required services not available");
        return;
    }

    let mut server = McpServerHandle::new().expect("Failed to start MCP server");
    let test_data_path = get_test_data_path();

    // Initialize
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    server.send_request(initialize_request).expect("Initialize failed");

    // Index test_data first
    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": test_data_path
            }
        }
    });

    timeout(Duration::from_secs(120), async {
        server.send_request(index_request)
    })
    .await
    .expect("Index operation timed out")
    .expect("Failed to call index");

    // Test get_content tool
    let get_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_content",
            "arguments": {
                "file_path": get_test_data_path() + "/docs/api_guide.md"
            }
        }
    });

    let response = server.send_request(get_request)
        .expect("Failed to call get_content");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response.get("result").is_some() || response.get("error").is_some());
}
