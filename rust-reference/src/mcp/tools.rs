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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_tool_schema() {
        let tool = McpTool::index_tool();

        assert_eq!(tool.name, "index");
        assert_eq!(tool.description, "Index directories for semantic search");

        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["properties"]["directory_path"].is_object());
        assert_eq!(schema["properties"]["directory_path"]["type"], "string");
        assert!(schema["required"].is_array());
        assert_eq!(schema["required"][0], "directory_path");
    }

    #[test]
    fn test_search_tool_schema() {
        let tool = McpTool::search_tool();

        assert_eq!(tool.name, "search");
        assert_eq!(tool.description, "Search indexed content semantically");

        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");

        let properties = &schema["properties"];
        assert!(properties["query"].is_object());
        assert_eq!(properties["query"]["type"], "string");

        assert!(properties["directory_path"].is_object());
        assert_eq!(properties["directory_path"]["type"], "string");

        assert!(properties["limit"].is_object());
        assert_eq!(properties["limit"]["type"], "integer");
        assert_eq!(properties["limit"]["default"], 10);

        assert_eq!(schema["required"][0], "query");
    }

    #[test]
    fn test_similar_files_tool_schema() {
        let tool = McpTool::similar_files_tool();

        assert_eq!(tool.name, "similar_files");
        assert_eq!(tool.description, "Find files similar to a given file");

        let schema = &tool.input_schema;
        let properties = &schema["properties"];

        assert!(properties["file_path"].is_object());
        assert_eq!(properties["file_path"]["type"], "string");

        assert!(properties["limit"].is_object());
        assert_eq!(properties["limit"]["type"], "integer");
        assert_eq!(properties["limit"]["default"], 10);

        assert_eq!(schema["required"][0], "file_path");
    }

    #[test]
    fn test_get_content_tool_schema() {
        let tool = McpTool::get_content_tool();

        assert_eq!(tool.name, "get_content");
        assert_eq!(
            tool.description,
            "Get file content with optional chunk selection"
        );

        let schema = &tool.input_schema;
        let properties = &schema["properties"];

        assert!(properties["file_path"].is_object());
        assert_eq!(properties["file_path"]["type"], "string");

        assert!(properties["chunks"].is_object());
        assert_eq!(properties["chunks"]["type"], "string");

        assert_eq!(schema["required"][0], "file_path");
    }

    #[test]
    fn test_server_info_tool_schema() {
        let tool = McpTool::server_info_tool();

        assert_eq!(tool.name, "server_info");
        assert_eq!(tool.description, "Get server information and statistics");

        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["properties"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_all_tools_completeness() {
        let all_tools = McpTool::all_tools();

        assert_eq!(all_tools.len(), 5);

        let tool_names: Vec<&str> = all_tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"index"));
        assert!(tool_names.contains(&"search"));
        assert!(tool_names.contains(&"similar_files"));
        assert!(tool_names.contains(&"get_content"));
        assert!(tool_names.contains(&"server_info"));
    }

    #[test]
    fn test_tool_serialization() {
        let tool = McpTool::index_tool();
        let serialized = serde_json::to_string(&tool).unwrap();

        assert!(serialized.contains(r#""name":"index""#));
        assert!(serialized.contains(r#""description":"Index directories for semantic search""#));
        assert!(serialized.contains(r#""input_schema""#));
    }

    #[test]
    fn test_tool_deserialization() {
        let json_str = r#"{
            "name": "test_tool",
            "description": "A test tool",
            "input_schema": {
                "type": "object",
                "properties": {
                    "param": {"type": "string"}
                },
                "required": ["param"]
            }
        }"#;

        let tool: McpTool = serde_json::from_str(json_str).unwrap();

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.input_schema["type"], "object");
        assert_eq!(tool.input_schema["properties"]["param"]["type"], "string");
    }

    #[test]
    fn test_schema_validation_structure() {
        let tools = McpTool::all_tools();

        for tool in tools {
            // All tools should have valid JSON schema structure
            assert!(tool.input_schema.is_object());
            assert_eq!(tool.input_schema["type"], "object");
            assert!(tool.input_schema["properties"].is_object());

            // All tools should have non-empty names and descriptions
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());

            // Required fields should be arrays if present
            if let Some(required) = tool.input_schema.get("required") {
                assert!(required.is_array());
            }
        }
    }

    #[test]
    fn test_tool_parameter_descriptions() {
        let tools = McpTool::all_tools();

        for tool in tools {
            let properties = tool.input_schema["properties"].as_object().unwrap();

            // All parameters should have descriptions
            for (param_name, param_schema) in properties {
                assert!(
                    param_schema.get("description").is_some(),
                    "Parameter '{param_name}' in tool '{}' missing description",
                    tool.name
                );

                let description = param_schema["description"].as_str().unwrap();
                assert!(
                    !description.is_empty(),
                    "Parameter '{param_name}' in tool '{}' has empty description",
                    tool.name
                );
            }
        }
    }

    #[test]
    fn test_required_parameters_exist() {
        let tools = McpTool::all_tools();

        for tool in tools {
            if let Some(required) = tool.input_schema.get("required") {
                let required_array = required.as_array().unwrap();
                let properties = tool.input_schema["properties"].as_object().unwrap();

                for required_param in required_array {
                    let param_name = required_param.as_str().unwrap();
                    assert!(
                        properties.contains_key(param_name),
                        "Required parameter '{param_name}' not defined in properties for tool '{}'",
                        tool.name
                    );
                }
            }
        }
    }

    #[test]
    fn test_tool_clone_and_debug() {
        let tool = McpTool::index_tool();
        let cloned = tool.clone();

        assert_eq!(tool.name, cloned.name);
        assert_eq!(tool.description, cloned.description);
        assert_eq!(tool.input_schema, cloned.input_schema);

        // Test Debug trait
        let debug_output = format!("{tool:?}");
        assert!(debug_output.contains("McpTool"));
        assert!(debug_output.contains("index"));
    }

    #[test]
    fn test_specific_tool_parameters() {
        // Test index tool has correct parameter
        let index_tool = McpTool::index_tool();
        let props = index_tool.input_schema["properties"].as_object().unwrap();
        assert!(props.contains_key("directory_path"));
        assert!(
            index_tool.input_schema["properties"]["directory_path"]["description"]
                .as_str()
                .unwrap()
                .contains("comma-separated")
        );

        // Test search tool has optional parameters
        let search_tool = McpTool::search_tool();
        let search_props = search_tool.input_schema["properties"].as_object().unwrap();
        assert!(search_props.contains_key("query"));
        assert!(search_props.contains_key("directory_path"));
        assert!(search_props.contains_key("limit"));

        let required = search_tool.input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "query");

        // Test get_content tool has optional chunks parameter
        let get_tool = McpTool::get_content_tool();
        let get_props = get_tool.input_schema["properties"].as_object().unwrap();
        assert!(get_props.contains_key("file_path"));
        assert!(get_props.contains_key("chunks"));

        let get_required = get_tool.input_schema["required"].as_array().unwrap();
        assert_eq!(get_required.len(), 1);
        assert_eq!(get_required[0], "file_path");
    }
}
