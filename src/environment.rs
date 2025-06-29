use crate::{Config, Result};
use log::{debug, info};

pub async fn validate_environment(config: &Config) -> Result<()> {
    // Test Qdrant connection
    let qdrant_url = format!("{}/collections", config.storage.qdrant.endpoint);
    match reqwest::Client::new().get(&qdrant_url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(crate::error::IndexerError::environment_setup(format!(
                    "Qdrant not accessible at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                    config.storage.qdrant.endpoint
                )));
            }
            info!("Connected to Qdrant at {}", config.storage.qdrant.endpoint);
        }
        Err(_) => {
            return Err(crate::error::IndexerError::environment_setup(format!(
                "Cannot connect to Qdrant at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                config.storage.qdrant.endpoint
            )));
        }
    }

    // Test embedding provider
    match config.embedding.provider.as_str() {
        "ollama" => {
            let ollama_url = format!("{}/api/tags", config.embedding.endpoint);
            match reqwest::Client::new().get(&ollama_url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Err(crate::error::IndexerError::environment_setup(format!(
                            "Ollama not accessible at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                            config.embedding.endpoint
                        )));
                    }
                    info!("Connected to Ollama at {}", config.embedding.endpoint);
                }
                Err(_) => {
                    return Err(crate::error::IndexerError::environment_setup(format!(
                        "Cannot connect to Ollama at {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                        config.embedding.endpoint
                    )));
                }
            }
        }
        "openai" => {
            if config.embedding.api_key.is_none() {
                return Err(crate::error::IndexerError::environment_setup(
                    "OpenAI API key required but not provided (set OPENAI_API_KEY). Setup required: https://github.com/peteretelej/directory-indexer#setup".to_string()
                ));
            }
            info!("OpenAI API key configured for embeddings");
        }
        _ => {
            return Err(crate::error::IndexerError::environment_setup(format!(
                "Unknown embedding provider: {}. Setup required: https://github.com/peteretelej/directory-indexer#setup",
                config.embedding.provider
            )));
        }
    }

    debug!("Environment validation passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{
        Config, EmbeddingConfig, IndexingConfig, MonitoringConfig, QdrantConfig, StorageConfig,
    };
    use tokio;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config_with_endpoints(
        qdrant_endpoint: String,
        embedding_endpoint: String,
    ) -> Config {
        Config {
            storage: StorageConfig {
                sqlite_path: std::path::PathBuf::from("/tmp/test.db"),
                qdrant: QdrantConfig {
                    endpoint: qdrant_endpoint,
                    collection: "test".to_string(),
                    api_key: None,
                },
            },
            embedding: EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                endpoint: embedding_endpoint,
                api_key: None,
            },
            indexing: IndexingConfig {
                chunk_size: 512,
                overlap: 50,
                max_file_size: 1024 * 1024,
                ignore_patterns: vec![],
                concurrency: 1,
            },
            monitoring: MonitoringConfig {
                file_watching: false,
                batch_size: 10,
            },
        }
    }

    #[tokio::test]
    async fn test_validate_environment_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"collections": []})),
            )
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"models": []})),
            )
            .mount(&server)
            .await;

        let config = create_test_config_with_endpoints(server.uri(), server.uri());

        let result = validate_environment(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_environment_qdrant_unreachable() {
        let config = create_test_config_with_endpoints(
            "http://localhost:99999".to_string(),
            "http://localhost:11434".to_string(),
        );

        let result = validate_environment(&config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            crate::error::IndexerError::EnvironmentSetup { message } => {
                assert!(message.contains("Cannot connect to Qdrant"));
                assert!(message.contains("http://localhost:99999"));
                assert!(message.contains("Setup required"));
            }
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }

    #[tokio::test]
    async fn test_validate_environment_qdrant_error_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let config =
            create_test_config_with_endpoints(server.uri(), "http://localhost:11434".to_string());

        let result = validate_environment(&config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            crate::error::IndexerError::EnvironmentSetup { message } => {
                assert!(message.contains("Qdrant not accessible"));
                assert!(message.contains("Setup required"));
            }
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }

    #[tokio::test]
    async fn test_validate_environment_ollama_unreachable() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"collections": []})),
            )
            .mount(&server)
            .await;

        let config =
            create_test_config_with_endpoints(server.uri(), "http://localhost:99999".to_string());

        let result = validate_environment(&config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            crate::error::IndexerError::EnvironmentSetup { message } => {
                assert!(message.contains("Cannot connect to Ollama"));
                assert!(message.contains("http://localhost:99999"));
                assert!(message.contains("Setup required"));
            }
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }

    #[tokio::test]
    async fn test_validate_environment_ollama_error_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"collections": []})),
            )
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let config = create_test_config_with_endpoints(server.uri(), server.uri());

        let result = validate_environment(&config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            crate::error::IndexerError::EnvironmentSetup { message } => {
                assert!(message.contains("Ollama not accessible"));
                assert!(message.contains("Setup required"));
            }
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }

    #[tokio::test]
    async fn test_validate_environment_openai_with_api_key() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"collections": []})),
            )
            .mount(&server)
            .await;

        let mut config =
            create_test_config_with_endpoints(server.uri(), "http://api.openai.com".to_string());
        config.embedding.provider = "openai".to_string();
        config.embedding.api_key = Some("test-api-key".to_string());

        let result = validate_environment(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_environment_openai_missing_api_key() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"collections": []})),
            )
            .mount(&server)
            .await;

        let mut config =
            create_test_config_with_endpoints(server.uri(), "http://api.openai.com".to_string());
        config.embedding.provider = "openai".to_string();
        config.embedding.api_key = None;

        let result = validate_environment(&config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            crate::error::IndexerError::EnvironmentSetup { message } => {
                assert!(message.contains("OpenAI API key required"));
                assert!(message.contains("OPENAI_API_KEY"));
                assert!(message.contains("Setup required"));
            }
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }

    #[tokio::test]
    async fn test_validate_environment_unknown_provider() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"collections": []})),
            )
            .mount(&server)
            .await;

        let mut config = create_test_config_with_endpoints(
            server.uri(),
            "http://unknown-provider.com".to_string(),
        );
        config.embedding.provider = "unknown".to_string();

        let result = validate_environment(&config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            crate::error::IndexerError::EnvironmentSetup { message } => {
                assert!(message.contains("Unknown embedding provider: unknown"));
                assert!(message.contains("Setup required"));
            }
            _ => panic!("Expected EnvironmentSetup error"),
        }
    }
}
