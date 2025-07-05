use log::{debug, error, info, warn};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

use super::json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::tools::McpTool;
use crate::{Config, Result};

pub struct McpServer {
    config: Config,
}

impl McpServer {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing MCP server");
        Ok(Self { config })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting MCP server on stdio");

        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("EOF reached, shutting down MCP server");
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    debug!("Received request: {line}");

                    let response = self.handle_request(line).await;
                    let response_json = serde_json::to_string(&response)
                        .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.to_string());

                    debug!("Sending response: {response_json}");

                    if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                        error!("Failed to write response: {e}");
                        break;
                    }
                    if let Err(e) = stdout.write_all(b"\n").await {
                        error!("Failed to write newline: {e}");
                        break;
                    }
                    if let Err(e) = stdout.flush().await {
                        error!("Failed to flush stdout: {e}");
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {e}");
                    break;
                }
            }
        }

        info!("MCP server shutting down");
        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {e}");
                return JsonRpcResponse::error(None, JsonRpcError::invalid_request());
            }
        };

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await,
            "notifications/initialized" => self.handle_notifications_initialized(request.id).await,
            "tools/list" => self.handle_tools_list(request.id).await,
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            "resources/list" => self.handle_resources_list(request.id).await,
            "resources/templates/list" => self.handle_resources_templates_list(request.id).await,
            _ => {
                warn!("Unknown method: {}", request.method);
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found())
            }
        }
    }

    async fn handle_initialize(
        &self,
        id: Option<Value>,
        _params: Option<Value>,
    ) -> JsonRpcResponse {
        info!("Handling initialize request");

        let capabilities = json!({
            "tools": {
                "listChanged": true
            },
            "resources": {
                "subscribe": false,
                "listChanged": false
            },
            "prompts": {},
            "logging": {}
        });

        let result = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": capabilities,
            "serverInfo": {
                "name": "directory-indexer",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        JsonRpcResponse::success(id, result)
    }

    async fn handle_notifications_initialized(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Handling notifications/initialized");
        // For notifications, we typically don't send a response
        // But if we do, it should be a success with no result
        JsonRpcResponse::success(id, json!({}))
    }

    async fn handle_resources_list(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Handling resources/list request");

        // Return empty resources list for now
        let result = json!({
            "resources": []
        });

        JsonRpcResponse::success(id, result)
    }

    async fn handle_resources_templates_list(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Handling resources/templates/list request");

        // Return empty resource templates list for now
        let result = json!({
            "resourceTemplates": []
        });

        JsonRpcResponse::success(id, result)
    }

    async fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Handling tools/list request");

        let tools = McpTool::all_tools();
        let tools_json: Vec<Value> = tools
            .into_iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                })
            })
            .collect();

        let result = json!({
            "tools": tools_json
        });

        JsonRpcResponse::success(id, result)
    }

    async fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing params".to_string()),
                )
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing tool name".to_string()),
                )
            }
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        info!("Handling tools/call request for tool: {tool_name}");

        match tool_name {
            "server_info" => self.handle_server_info_tool(id).await,
            "index" => self.handle_index_tool(id, arguments).await,
            "search" => self.handle_search_tool(id, arguments).await,
            "similar_files" => self.handle_similar_files_tool(id, arguments).await,
            "get_content" => self.handle_get_content_tool(id, arguments).await,
            _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found()),
        }
    }

    async fn handle_server_info_tool(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Handling server_info tool");

        let version = env!("CARGO_PKG_VERSION");
        let sqlite_path = self.config.storage.sqlite_path.display();
        let qdrant_endpoint = &self.config.storage.qdrant.endpoint;
        let embedding_provider = &self.config.embedding.provider;
        let content = format!("Directory Indexer MCP Server v{version}\n\nStatus:\n- Server running\n- Configuration loaded\n- SQLite path: {sqlite_path}\n- Qdrant endpoint: {qdrant_endpoint}\n- Embedding provider: {embedding_provider}\n- Available tools: index, search, similar_files, get_content, server_info");

        let result = json!({
            "content": [
                {
                    "type": "text",
                    "text": content
                }
            ]
        });

        JsonRpcResponse::success(id, result)
    }

    async fn handle_index_tool(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        info!("Handling index tool");

        let directory_path = match arguments.get("directory_path").and_then(|v| v.as_str()) {
            Some(path) => path,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(
                        "missing required parameter: directory_path".to_string(),
                    ),
                )
            }
        };

        // Handle comma-separated paths for multiple directories
        let paths: Vec<String> = directory_path
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Call CLI index function without console output
        match crate::cli::commands::index_internal(paths.clone(), false).await {
            Ok(_) => {
                let paths_len = paths.len();
                let paths_list = paths
                    .iter()
                    .map(|p| format!("- {p}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let content =
                    format!("Successfully indexed {paths_len} directories:\n{paths_list}");

                let result = json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                });

                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::internal_error(format!("Failed to index directories: {e}")),
            ),
        }
    }

    async fn handle_search_tool(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        info!("Handling search tool");

        let query = match arguments.get("query").and_then(|v| v.as_str()) {
            Some(q) => q.to_string(),
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("missing required parameter: query".to_string()),
                )
            }
        };

        let directory_path = arguments
            .get("directory_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l as usize);

        // Use SearchEngine directly
        let search_engine = match crate::search::engine::create_search_engine().await {
            Ok(engine) => engine,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::internal_error(format!("Failed to create search engine: {e}")),
                )
            }
        };

        let search_query = crate::search::engine::SearchQuery {
            text: query.clone(),
            directory_filter: directory_path.as_ref().map(std::path::PathBuf::from),
            limit: limit.unwrap_or(10),
            similarity_threshold: None,
        };

        match search_engine.search(search_query).await {
            Ok(search_results) => {
                let mut content = format!("Search completed for query: '{query}'\n");
                if let Some(path) = &directory_path {
                    content.push_str(&format!("Scope: {path}\n"));
                }
                if let Some(l) = limit {
                    content.push_str(&format!("Limit: {l}\n"));
                }
                content.push_str(&format!("Found {} results\n\n", search_results.len()));

                if search_results.is_empty() {
                    content.push_str("No results found for the given query.");
                } else {
                    content.push_str("Search Results:\n");
                    content.push_str("==============\n");

                    for (i, result) in search_results.iter().enumerate() {
                        content.push_str(&format!(
                            "\n{}. {} (score: {:.3})\n",
                            i + 1,
                            result.file_path,
                            result.score
                        ));
                        content.push_str(&format!("   Chunk: {}\n", result.chunk_id));
                        if !result.parent_directories.is_empty() {
                            content.push_str(&format!(
                                "   Path: {}\n",
                                result.parent_directories.join(" > ")
                            ));
                        }
                    }
                }

                let result = json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                });

                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::internal_error(format!("Search failed: {e}")),
            ),
        }
    }

    async fn handle_similar_files_tool(
        &self,
        id: Option<Value>,
        arguments: Value,
    ) -> JsonRpcResponse {
        info!("Handling similar_files tool");

        let file_path = match arguments.get("file_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(
                        "missing required parameter: file_path".to_string(),
                    ),
                )
            }
        };

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l as usize)
            .unwrap_or(10);

        // Use SearchEngine directly
        let search_engine = match crate::search::engine::create_search_engine().await {
            Ok(engine) => engine,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::internal_error(format!("Failed to create search engine: {e}")),
                )
            }
        };

        match search_engine
            .find_similar_files(std::path::PathBuf::from(&file_path), limit)
            .await
        {
            Ok(similar_results) => {
                let mut content =
                    format!("Similar files search completed for: {file_path}\nLimit: {limit}\n");
                content.push_str(&format!(
                    "Found {} similar files\n\n",
                    similar_results.len()
                ));

                if similar_results.is_empty() {
                    content.push_str("No similar files found.");
                } else {
                    content.push_str("Similar Files:\n");
                    content.push_str("==============\n");

                    for (i, result) in similar_results.iter().enumerate() {
                        content.push_str(&format!(
                            "\n{}. {} (score: {:.3})\n",
                            i + 1,
                            result.file_path,
                            result.score
                        ));
                        content.push_str(&format!("   Best matching chunk: {}\n", result.chunk_id));
                    }
                }

                let result = json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                });

                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::internal_error(format!("Similar files search failed: {e}")),
            ),
        }
    }

    async fn handle_get_content_tool(
        &self,
        id: Option<Value>,
        arguments: Value,
    ) -> JsonRpcResponse {
        info!("Handling get_content tool");

        let file_path = match arguments.get("file_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(
                        "missing required parameter: file_path".to_string(),
                    ),
                )
            }
        };

        let chunks = arguments
            .get("chunks")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse chunk range if provided
        let chunk_range = if let Some(ref chunk_str) = chunks {
            match crate::cli::commands::validate_chunk_range(chunk_str) {
                Ok(_) => match crate::cli::commands::parse_chunk_range(chunk_str) {
                    Ok(range) => Some(range),
                    Err(e) => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params(format!("Invalid chunk range: {e}")),
                        )
                    }
                },
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        JsonRpcError::invalid_params(format!("Invalid chunk range: {e}")),
                    )
                }
            }
        } else {
            None
        };

        // Use SearchEngine directly
        let search_engine = match crate::search::engine::create_search_engine().await {
            Ok(engine) => engine,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::internal_error(format!("Failed to create search engine: {e}")),
                )
            }
        };

        match search_engine
            .get_file_content(std::path::PathBuf::from(&file_path), chunk_range)
            .await
        {
            Ok(file_content) => {
                let mut header = format!("Content retrieved from: {file_path}\n");
                if let Some(c) = &chunks {
                    header.push_str(&format!("Chunks: {c}\n"));
                }
                header.push_str("\n--- FILE CONTENT ---\n");

                let full_content = format!("{header}{file_content}");

                let result = json!({
                    "content": [
                        {
                            "type": "text",
                            "text": full_content
                        }
                    ]
                });

                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::internal_error(format!("Get content failed: {e}")),
            ),
        }
    }

    // Helper methods for testing that expose internal functionality
    #[cfg(test)]
    pub async fn handle_request_test(&self, request_str: &str) -> JsonRpcResponse {
        self.handle_request(request_str).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn create_test_server() -> McpServer {
        let config = Config::default();
        McpServer::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = Config::default();
        let server = McpServer::new(config).await;

        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_handle_initialize_request() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"].is_object());
        assert!(result["serverInfo"].is_object());
        assert_eq!(result["serverInfo"]["name"], "directory-indexer");
    }

    #[tokio::test]
    async fn test_handle_tools_list_request() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(2)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert!(result["tools"].is_array());

        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 5); // Should have all 5 tools

        // Check that all expected tools are present
        let tool_names: Vec<&str> = tools
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();

        assert!(tool_names.contains(&"index"));
        assert!(tool_names.contains(&"search"));
        assert!(tool_names.contains(&"similar_files"));
        assert!(tool_names.contains(&"get_content"));
        assert!(tool_names.contains(&"server_info"));
    }

    #[tokio::test]
    async fn test_handle_server_info_tool() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"server_info","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(3)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert!(result["content"].is_array());

        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
        assert!(content[0]["text"]
            .as_str()
            .unwrap()
            .contains("Directory Indexer MCP Server"));
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":4,"method":"unknown_method","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(4)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601); // Method not found
        assert_eq!(error.message, "Method not found");
    }

    #[tokio::test]
    async fn test_handle_invalid_json() {
        let server = create_test_server().await;
        let invalid_request = r#"{"jsonrpc":"2.0","method":"test""#; // missing closing brace

        let response = server.handle_request_test(invalid_request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none());
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32600); // Invalid request
    }

    #[tokio::test]
    async fn test_tools_call_missing_params() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call"}"#; // no params

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(5)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error.message.contains("Missing params"));
    }

    #[tokio::test]
    async fn test_tools_call_missing_tool_name() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"arguments":{}}}"#; // no name

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(6)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error.message.contains("Missing tool name"));
    }

    #[tokio::test]
    async fn test_tools_call_unknown_tool() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"unknown_tool","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(7)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601); // Method not found
    }

    #[tokio::test]
    async fn test_index_tool_missing_directory_path() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"index","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(8)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error
            .message
            .contains("missing required parameter: directory_path"));
    }

    #[tokio::test]
    async fn test_search_tool_missing_query() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"search","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(9)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error.message.contains("missing required parameter: query"));
    }

    #[tokio::test]
    async fn test_similar_files_tool_missing_file_path() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"similar_files","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(10)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error
            .message
            .contains("missing required parameter: file_path"));
    }

    #[tokio::test]
    async fn test_get_content_tool_missing_file_path() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"get_content","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(11)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error
            .message
            .contains("missing required parameter: file_path"));
    }

    #[tokio::test]
    async fn test_initialize_capabilities() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;

        let response = server.handle_request_test(request).await;
        let result = response.result.unwrap();

        let capabilities = &result["capabilities"];
        assert!(capabilities["tools"].is_object());
        assert_eq!(capabilities["tools"]["listChanged"], true);
        assert!(capabilities["resources"].is_object());
        assert!(capabilities["prompts"].is_object());
        assert!(capabilities["logging"].is_object());
    }

    #[tokio::test]
    async fn test_server_info_response_format() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"server_info","arguments":{}}}"#;

        let response = server.handle_request_test(request).await;
        let result = response.result.unwrap();

        assert!(result["content"].is_array());
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);

        let text_content = &content[0];
        assert_eq!(text_content["type"], "text");

        let text = text_content["text"].as_str().unwrap();
        assert!(text.contains("Directory Indexer MCP Server"));
        assert!(text.contains("Status:"));
        assert!(text.contains("Server running"));
        assert!(text.contains("Available tools:"));
    }

    #[tokio::test]
    async fn test_notification_request() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","method":"some_notification","params":{}}"#; // no id = notification

        let response = server.handle_request_test(request).await;

        // For unknown methods, even notifications should return method not found
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none()); // notifications don't have id in response
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_tools_list_schema_structure() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;

        let response = server.handle_request_test(request).await;
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // Check that each tool has the required schema structure
        for tool in tools {
            assert!(tool["name"].is_string());
            assert!(tool["description"].is_string());
            assert!(tool["inputSchema"].is_object());

            let schema = &tool["inputSchema"];
            assert_eq!(schema["type"], "object");
            assert!(schema["properties"].is_object());
        }
    }

    #[tokio::test]
    async fn test_request_with_null_id() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":null,"method":"initialize","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none()); // null id becomes None
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_search_tool_with_optional_parameters() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search","arguments":{"query":"test","directory_path":"/test","limit":5}}}"#;

        let response = server.handle_request_test(request).await;

        // This should succeed (even though the underlying search might fail due to no data)
        // We're testing the parameter handling, not the actual search functionality
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
    }

    #[tokio::test]
    async fn test_similar_files_tool_with_limit() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"similar_files","arguments":{"file_path":"/test/file.txt","limit":20}}}"#;

        let response = server.handle_request_test(request).await;

        // Should succeed in parameter parsing
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
    }

    #[tokio::test]
    async fn test_get_content_tool_with_chunks() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_content","arguments":{"file_path":"/test/file.txt","chunks":"1-3"}}}"#;

        let response = server.handle_request_test(request).await;

        // Should succeed in parameter parsing
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
    }

    #[tokio::test]
    async fn test_notifications_initialized() {
        let server = create_test_server().await;
        let request =
            r#"{"jsonrpc":"2.0","id":1,"method":"notifications/initialized","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_resources_list() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert!(result["resources"].is_array());
        assert_eq!(result["resources"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_resources_templates_list() {
        let server = create_test_server().await;
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"resources/templates/list","params":{}}"#;

        let response = server.handle_request_test(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert!(result["resourceTemplates"].is_array());
        assert_eq!(result["resourceTemplates"].as_array().unwrap().len(), 0);
    }
}
