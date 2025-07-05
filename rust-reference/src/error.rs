use thiserror::Error;

pub type Result<T> = std::result::Result<T, IndexerError>;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Embedding provider error: {message}")]
    Embedding { message: String },

    #[error("Vector store error: {message}")]
    VectorStore { message: String },

    #[error("File processing error: {message}")]
    FileProcessing { message: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Resource not found: {message}")]
    NotFound { message: String },

    #[error("MCP protocol error: {message}")]
    Mcp { message: String },

    #[error("Environment setup required: {message}")]
    EnvironmentSetup { message: String },
}

impl IndexerError {
    pub fn embedding<S: Into<String>>(message: S) -> Self {
        Self::Embedding {
            message: message.into(),
        }
    }

    pub fn vector_store<S: Into<String>>(message: S) -> Self {
        Self::VectorStore {
            message: message.into(),
        }
    }

    pub fn file_processing<S: Into<String>>(message: S) -> Self {
        Self::FileProcessing {
            message: message.into(),
        }
    }

    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    pub fn mcp<S: Into<String>>(message: S) -> Self {
        Self::Mcp {
            message: message.into(),
        }
    }

    pub fn environment_setup<S: Into<String>>(message: S) -> Self {
        Self::EnvironmentSetup {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display_messages() {
        let io_error = IndexerError::Io(io::Error::new(io::ErrorKind::NotFound, "test io error"));
        assert!(io_error.to_string().contains("IO error"));
        assert!(io_error.to_string().contains("test io error"));

        let embedding_error = IndexerError::embedding("embedding failed");
        assert_eq!(
            embedding_error.to_string(),
            "Embedding provider error: embedding failed"
        );

        let vector_error = IndexerError::vector_store("qdrant unavailable");
        assert_eq!(
            vector_error.to_string(),
            "Vector store error: qdrant unavailable"
        );

        let file_error = IndexerError::file_processing("cannot read file");
        assert_eq!(
            file_error.to_string(),
            "File processing error: cannot read file"
        );

        let input_error = IndexerError::invalid_input("invalid path");
        assert_eq!(input_error.to_string(), "Invalid input: invalid path");

        let not_found_error = IndexerError::not_found("file not found");
        assert_eq!(
            not_found_error.to_string(),
            "Resource not found: file not found"
        );

        let mcp_error = IndexerError::mcp("protocol error");
        assert_eq!(mcp_error.to_string(), "MCP protocol error: protocol error");

        let env_error = IndexerError::environment_setup("qdrant not running");
        assert_eq!(
            env_error.to_string(),
            "Environment setup required: qdrant not running"
        );
    }

    #[test]
    fn test_error_constructor_functions() {
        let embedding_error = IndexerError::embedding("test message");
        match embedding_error {
            IndexerError::Embedding { message } => assert_eq!(message, "test message"),
            _ => panic!("Expected Embedding error"),
        }

        let vector_error = IndexerError::vector_store("qdrant error");
        match vector_error {
            IndexerError::VectorStore { message } => assert_eq!(message, "qdrant error"),
            _ => panic!("Expected VectorStore error"),
        }

        let file_error = IndexerError::file_processing("read failed");
        match file_error {
            IndexerError::FileProcessing { message } => assert_eq!(message, "read failed"),
            _ => panic!("Expected FileProcessing error"),
        }

        let input_error = IndexerError::invalid_input("bad input");
        match input_error {
            IndexerError::InvalidInput { message } => assert_eq!(message, "bad input"),
            _ => panic!("Expected InvalidInput error"),
        }

        let not_found_error = IndexerError::not_found("missing");
        match not_found_error {
            IndexerError::NotFound { message } => assert_eq!(message, "missing"),
            _ => panic!("Expected NotFound error"),
        }

        let mcp_error = IndexerError::mcp("rpc error");
        match mcp_error {
            IndexerError::Mcp { message } => assert_eq!(message, "rpc error"),
            _ => panic!("Expected Mcp error"),
        }

        let env_error = IndexerError::environment_setup("setup needed");
        match env_error {
            IndexerError::EnvironmentSetup { message } => assert_eq!(message, "setup needed"),
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }

    #[test]
    fn test_error_from_conversions() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let indexer_error: IndexerError = io_error.into();
        match indexer_error {
            IndexerError::Io(_) => {}
            _ => panic!("Expected Io error from conversion"),
        }

        let json_error = serde_json::from_str::<i32>("invalid json").unwrap_err();
        let indexer_error: IndexerError = json_error.into();
        match indexer_error {
            IndexerError::Json(_) => {}
            _ => panic!("Expected Json error from conversion"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = IndexerError::embedding("debug test");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Embedding"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_error_string_conversion() {
        let error1 = IndexerError::embedding(String::from("owned string"));
        let error2 = IndexerError::embedding("borrowed string");

        match (&error1, &error2) {
            (
                IndexerError::Embedding { message: msg1 },
                IndexerError::Embedding { message: msg2 },
            ) => {
                assert_eq!(msg1, "owned string");
                assert_eq!(msg2, "borrowed string");
            }
            _ => panic!("Expected Embedding errors"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        fn returns_error() -> Result<i32> {
            Err(IndexerError::invalid_input("test error"))
        }

        assert_eq!(returns_result().unwrap(), 42);
        assert!(returns_error().is_err());
    }
}
