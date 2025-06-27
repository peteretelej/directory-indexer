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
}
