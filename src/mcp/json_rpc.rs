use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl JsonRpcError {
    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    pub fn invalid_params(message: String) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid params: {message}"),
            data: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            code: -32603,
            message: format!("Internal error: {message}"),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test_method".to_string(),
            params: Some(json!({"param": "value"})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let expected =
            r#"{"jsonrpc":"2.0","id":1,"method":"test_method","params":{"param":"value"}}"#;

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json_str =
            r#"{"jsonrpc":"2.0","id":1,"method":"test_method","params":{"param":"value"}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.method, "test_method");
        assert_eq!(request.params, Some(json!({"param": "value"})));
    }

    #[test]
    fn test_json_rpc_request_without_params() {
        let json_str = r#"{"jsonrpc":"2.0","id":"test","method":"no_params"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!("test")));
        assert_eq!(request.method, "no_params");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_json_rpc_request_without_id() {
        let json_str = r#"{"jsonrpc":"2.0","method":"notification","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert!(request.id.is_none());
        assert_eq!(request.method, "notification");
        assert_eq!(request.params, Some(json!({})));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"result": "success"}));

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert_eq!(response.result, Some(json!({"result": "success"})));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let error = JsonRpcError::method_not_found();
        let response = JsonRpcResponse::error(Some(json!("test")), error);

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!("test")));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let err = response.error.unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse::success(Some(json!(42)), json!("test_result"));
        let serialized = serde_json::to_string(&response).unwrap();

        assert!(serialized.contains(r#""jsonrpc":"2.0""#));
        assert!(serialized.contains(r#""id":42"#));
        assert!(serialized.contains(r#""result":"test_result""#));
        assert!(!serialized.contains("error"));
    }

    #[test]
    fn test_json_rpc_error_response_serialization() {
        let error = JsonRpcError::invalid_params("test error".to_string());
        let response = JsonRpcResponse::error(Some(json!(1)), error);
        let serialized = serde_json::to_string(&response).unwrap();

        assert!(serialized.contains(r#""jsonrpc":"2.0""#));
        assert!(serialized.contains(r#""id":1"#));
        assert!(serialized.contains(r#""error""#));
        assert!(serialized.contains(r#""code":-32602"#));
        assert!(serialized.contains("Invalid params: test error"));
        assert!(!serialized.contains("result"));
    }

    #[test]
    fn test_json_rpc_error_codes() {
        let invalid_request = JsonRpcError::invalid_request();
        assert_eq!(invalid_request.code, -32600);
        assert_eq!(invalid_request.message, "Invalid Request");
        assert!(invalid_request.data.is_none());

        let method_not_found = JsonRpcError::method_not_found();
        assert_eq!(method_not_found.code, -32601);
        assert_eq!(method_not_found.message, "Method not found");

        let invalid_params = JsonRpcError::invalid_params("missing param".to_string());
        assert_eq!(invalid_params.code, -32602);
        assert_eq!(invalid_params.message, "Invalid params: missing param");

        let internal_error = JsonRpcError::internal_error("server error".to_string());
        assert_eq!(internal_error.code, -32603);
        assert_eq!(internal_error.message, "Internal error: server error");
    }

    #[test]
    fn test_json_rpc_error_with_data() {
        let mut error = JsonRpcError::invalid_params("test".to_string());
        error.data = Some(json!({"additional": "info"}));

        let response = JsonRpcResponse::error(Some(json!(1)), error);
        let serialized = serde_json::to_string(&response).unwrap();

        assert!(serialized.contains(r#""data":{"additional":"info"}"#));
    }

    #[test]
    fn test_json_rpc_protocol_compliance() {
        // Test that all responses have required jsonrpc field
        let success_response = JsonRpcResponse::success(None, json!("result"));
        assert_eq!(success_response.jsonrpc, "2.0");

        let error_response =
            JsonRpcResponse::error(None, JsonRpcError::internal_error("test".to_string()));
        assert_eq!(error_response.jsonrpc, "2.0");
    }

    #[test]
    fn test_invalid_json_deserialization() {
        // Test malformed JSON
        let invalid_json = r#"{"jsonrpc":"2.0","method":"test""#; // missing closing brace
        let result = serde_json::from_str::<JsonRpcRequest>(invalid_json);
        assert!(result.is_err());

        // Test missing required fields
        let missing_method = r#"{"jsonrpc":"2.0","id":1}"#;
        let result = serde_json::from_str::<JsonRpcRequest>(missing_method);
        assert!(result.is_err());

        let missing_jsonrpc = r#"{"id":1,"method":"test"}"#;
        let result = serde_json::from_str::<JsonRpcRequest>(missing_jsonrpc);
        assert!(result.is_err());
    }

    #[test]
    fn test_various_id_types() {
        // String ID
        let request_str = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!("string-id")),
            method: "test".to_string(),
            params: None,
        };

        // Number ID
        let request_num = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(42)),
            method: "test".to_string(),
            params: None,
        };

        // Test string and number IDs serialize/deserialize correctly
        for req in [request_str, request_num] {
            let serialized = serde_json::to_string(&req).unwrap();
            let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
            assert_eq!(req.id, deserialized.id);
        }
    }

    #[test]
    fn test_null_id_handling() {
        // Test explicit null ID in JSON
        let json_with_null_id = r#"{"jsonrpc":"2.0","id":null,"method":"test"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_with_null_id).unwrap();

        // NOTE: Serde deserializes JSON null as None for Option<Value>
        // This is standard serde behavior, not a bug
        assert!(request.id.is_none());
    }

    #[test]
    fn test_missing_id_handling() {
        // Test missing ID in JSON (notification)
        let json_without_id = r#"{"jsonrpc":"2.0","method":"notification"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_without_id).unwrap();

        // Missing ID should deserialize as None
        assert!(request.id.is_none());
    }

    #[test]
    fn test_complex_params() {
        let complex_params = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            },
            "boolean": true,
            "null_value": null
        });

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "complex_method".to_string(),
            params: Some(complex_params.clone()),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.params, deserialized.params);
    }
}
