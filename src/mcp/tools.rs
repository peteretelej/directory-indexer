use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl McpTool {
    pub fn index_tool() -> Self {
        Self {
            name: "index".to_string(),
            description: "Index directories for semantic search".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "directory_path": {
                        "type": "string",
                        "description": "Path to directory to index (or comma-separated paths)"
                    }
                },
                "required": ["directory_path"]
            }),
        }
    }

    pub fn search_tool() -> Self {
        Self {
            name: "search".to_string(),
            description: "Search indexed content semantically".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "directory_path": {
                        "type": "string",
                        "description": "Optional directory to scope search to"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        }
    }

    pub fn similar_files_tool() -> Self {
        Self {
            name: "similar_files".to_string(),
            description: "Find files similar to a given file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the reference file"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of similar files to return",
                        "default": 10
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    pub fn get_content_tool() -> Self {
        Self {
            name: "get_content".to_string(),
            description: "Get file content with optional chunk selection".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "chunks": {
                        "type": "string",
                        "description": "Optional chunk range (e.g., '2-5')"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    pub fn server_info_tool() -> Self {
        Self {
            name: "server_info".to_string(),
            description: "Get server information and statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    pub fn all_tools() -> Vec<Self> {
        vec![
            Self::index_tool(),
            Self::search_tool(),
            Self::similar_files_tool(),
            Self::get_content_tool(),
            Self::server_info_tool(),
        ]
    }
}
