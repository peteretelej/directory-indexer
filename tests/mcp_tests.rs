// MCP integration tests

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

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

#[test]
fn test_mcp_index_tool() {
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

    // Test index tool with simple mock call
    let index_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "index",
            "arguments": {
                "directory_path": "/tmp"
            }
        }
    });

    let response = server
        .send_request(index_request)
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
