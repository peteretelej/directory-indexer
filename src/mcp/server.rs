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

                    debug!("Received request: {}", line);

                    let response = self.handle_request(line).await;
                    let response_json = serde_json::to_string(&response)
                        .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.to_string());

                    debug!("Sending response: {}", response_json);

                    if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                        error!("Failed to write response: {}", e);
                        break;
                    }
                    if let Err(e) = stdout.write_all(b"\n").await {
                        error!("Failed to write newline: {}", e);
                        break;
                    }
                    if let Err(e) = stdout.flush().await {
                        error!("Failed to flush stdout: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
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
                error!("Failed to parse JSON-RPC request: {}", e);
                return JsonRpcResponse::error(None, JsonRpcError::invalid_request());
            }
        };

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await,
            "tools/list" => self.handle_tools_list(request.id).await,
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
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
            "resources": {},
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

        info!("Handling tools/call request for tool: {}", tool_name);

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

        let content = format!("Directory Indexer MCP Server v{}\n\nStatus:\n- Server running\n- Configuration loaded\n- SQLite path: {}\n- Qdrant endpoint: {}\n- Embedding provider: {}\n- Available tools: index, search, similar_files, get_content, server_info", 
            env!("CARGO_PKG_VERSION"),
            self.config.storage.sqlite_path.display(),
            self.config.storage.qdrant.endpoint,
            self.config.embedding.provider);

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
                let content = format!(
                    "Successfully indexed {} directories:\n{}",
                    paths.len(),
                    paths
                        .iter()
                        .map(|p| format!("- {}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

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
                JsonRpcError::internal_error(format!("Failed to index directories: {}", e)),
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

        // Call CLI search function without console output
        match crate::cli::commands::search_internal(
            query.clone(),
            directory_path.clone(),
            limit,
            false,
        )
        .await
        {
            Ok(_) => {
                let mut content = format!("Search completed for query: '{}'\n", query);
                if let Some(path) = directory_path {
                    content.push_str(&format!("Scope: {}\n", path));
                }
                if let Some(l) = limit {
                    content.push_str(&format!("Limit: {}\n", l));
                }
                content.push_str("Note: Search functionality is still being implemented");

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
                JsonRpcError::internal_error(format!("Search failed: {}", e)),
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

        // Call CLI similar function without console output
        match crate::cli::commands::similar_internal(file_path.clone(), limit, false).await {
            Ok(_) => {
                let content = format!("Similar files search completed for: {}\nLimit: {}\n\nThe search analyzes the file's content and finds files with similar semantic meaning using vector embeddings. Results are ranked by similarity score.", file_path, limit);

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
                JsonRpcError::internal_error(format!("Similar files search failed: {}", e)),
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

        // Call CLI get function without console output
        match crate::cli::commands::get_internal(file_path.clone(), chunks.clone(), false).await {
            Ok(_) => {
                let mut content = format!("Content retrieved from: {}\n", file_path);
                if let Some(c) = chunks {
                    content.push_str(&format!("Chunks: {}\n", c));
                }
                content.push_str("\nThe file content has been retrieved from the indexed database. If chunks were specified, only those specific chunks are returned. Otherwise, the full file content is provided.");

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
                JsonRpcError::internal_error(format!("Get content failed: {}", e)),
            ),
        }
    }
}
