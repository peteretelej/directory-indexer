use directory_indexer::{
    config::{
        settings::{
            EmbeddingConfig, IndexingConfig, MonitoringConfig, QdrantConfig, StorageConfig,
        },
        Config,
    },
    embedding::{ollama::OllamaProvider, EmbeddingProvider},
    health::check_system_health,
    storage::QdrantStore,
};
use tempfile::NamedTempFile;

// Helper to create test config with local services
fn create_test_config() -> Config {
    let temp_db = NamedTempFile::new().unwrap();

    // Generate unique collection name per test
    let test_collection = format!(
        "test-conn-{}",
        &uuid::Uuid::new_v4().to_string().replace('-', "")[..8]
    );

    let mut config = Config {
        storage: StorageConfig {
            sqlite_path: temp_db.path().to_path_buf(),
            qdrant: QdrantConfig {
                endpoint: "http://localhost:6333".to_string(),
                collection: test_collection,
                api_key: None,
            },
        },
        embedding: EmbeddingConfig {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
        },
        indexing: IndexingConfig {
            chunk_size: 256,
            overlap: 50,
            max_file_size: 1024 * 1024,
            ignore_patterns: vec![".git".to_string()],
            concurrency: 1,
        },
        monitoring: MonitoringConfig {
            file_watching: false,
            batch_size: 10,
        },
    };

    // Use the same environment variable override logic as Config::load()
    if let Ok(qdrant_endpoint) = std::env::var("QDRANT_ENDPOINT") {
        config.storage.qdrant.endpoint = qdrant_endpoint;
    }

    if let Ok(ollama_endpoint) = std::env::var("OLLAMA_ENDPOINT") {
        config.embedding.endpoint = ollama_endpoint;
    }

    config
}

#[tokio::test]
async fn test_system_health_check() {
    let config = create_test_config();

    println!("Testing system health with local services...");
    let health = check_system_health(&config).await;

    // Print detailed status for debugging
    println!("Health check results:");
    println!("  Ollama available: {}", health.ollama_available);
    println!("  Qdrant available: {}", health.qdrant_available);
    println!("  SQLite writable: {}", health.sqlite_writable);
    println!("  Available models: {:?}", health.ollama_models);

    // SQLite should always work with temp files
    assert!(health.sqlite_writable, "SQLite should be accessible");

    // Print warnings for external services if not available
    if !health.ollama_available {
        println!("⚠️  Ollama not available - make sure it's running: ollama serve");
        println!(
            "⚠️  Also ensure 'nomic-embed-text' model is installed: ollama pull nomic-embed-text"
        );
    }

    if !health.qdrant_available {
        println!("⚠️  Qdrant not available - make sure it's running: docker run -p 6333:6333 qdrant/qdrant");
    }
}

#[tokio::test]
async fn test_ollama_embedding_generation() {
    let config = create_test_config();

    // Skip test if Ollama is not available
    let health = check_system_health(&config).await;
    if !health.ollama_available {
        println!("Skipping Ollama test - service not available");
        return;
    }

    let provider = OllamaProvider::new(
        config.embedding.endpoint.clone(),
        config.embedding.model.clone(),
    );

    println!("Testing embedding generation...");
    let test_texts = vec![
        "This is a test document about machine learning.".to_string(),
        "Another test document about databases and indexing.".to_string(),
    ];

    match provider.generate_embeddings(test_texts.clone()).await {
        Ok(response) => {
            println!(
                "✓ Successfully generated {} embeddings",
                response.embeddings.len()
            );
            assert_eq!(response.embeddings.len(), 2);

            for (i, embedding) in response.embeddings.iter().enumerate() {
                println!("  Embedding {}: dimension {}", i, embedding.len());
                assert!(!embedding.is_empty(), "Embedding should not be empty");
            }

            println!("  Model: {}", response.model);
            if let Some(usage) = response.usage {
                println!("  Usage: {:?}", usage);
            }
        }
        Err(e) => {
            panic!("Failed to generate embeddings: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_operations() {
    let config = create_test_config();

    // Skip test if Qdrant is not available
    let health = check_system_health(&config).await;
    if !health.qdrant_available {
        println!("Skipping Qdrant test - service not available");
        return;
    }

    println!("Testing Qdrant operations...");

    let store = match QdrantStore::new_with_api_key(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
        config.storage.qdrant.api_key.clone(),
    )
    .await
    {
        Ok(store) => {
            println!("✓ Successfully connected to Qdrant");
            store
        }
        Err(e) => {
            panic!("Failed to connect to Qdrant: {}", e);
        }
    };

    // Test collection info
    match store.get_collection_info().await {
        Ok(info) => {
            println!(
                "✓ Collection info: {} points, {} indexed vectors",
                info.points_count, info.indexed_vectors_count
            );
        }
        Err(e) => {
            panic!("Failed to get collection info: {}", e);
        }
    }

    // Test health check
    match store.health_check().await {
        Ok(healthy) => {
            println!(
                "✓ Qdrant health check: {}",
                if healthy { "healthy" } else { "unhealthy" }
            );
            assert!(healthy, "Qdrant should be healthy");
        }
        Err(e) => {
            panic!("Failed to check Qdrant health: {}", e);
        }
    }
}

#[tokio::test]
async fn test_end_to_end_embedding_and_storage() {
    let config = create_test_config();

    // Check if both services are available
    let health = check_system_health(&config).await;
    if !health.ollama_available {
        println!("Skipping end-to-end test - Ollama not available");
        return;
    }
    if !health.qdrant_available {
        println!("Skipping end-to-end test - Qdrant not available");
        return;
    }

    println!("Testing end-to-end embedding generation and vector storage...");

    // Create providers
    let embedding_provider = OllamaProvider::new(
        config.embedding.endpoint.clone(),
        config.embedding.model.clone(),
    );

    // Clean up any existing test collection
    let cleanup_store = QdrantStore::new_with_api_key(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
        config.storage.qdrant.api_key.clone(),
    )
    .await;
    if let Ok(store) = cleanup_store {
        let _ = store.delete_collection().await; // Ignore errors if collection doesn't exist
    }

    let vector_store = QdrantStore::new_with_api_key(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
        config.storage.qdrant.api_key.clone(),
    )
    .await
    .expect("Failed to create Qdrant store");

    // Generate embeddings
    let test_texts = vec![
        "Machine learning algorithms for text processing".to_string(),
        "Database indexing and search optimization".to_string(),
    ];

    let embedding_response = embedding_provider
        .generate_embeddings(test_texts)
        .await
        .expect("Failed to generate embeddings");

    println!(
        "✓ Generated {} embeddings",
        embedding_response.embeddings.len()
    );

    // Create vector points
    use directory_indexer::storage::qdrant::VectorPoint;
    use uuid::Uuid;
    let points: Vec<VectorPoint> = embedding_response
        .embeddings
        .into_iter()
        .enumerate()
        .map(|(i, embedding)| VectorPoint {
            id: Uuid::new_v4().to_string(),
            vector: embedding,
            file_path: format!("/test/document_{}.txt", i),
            chunk_id: 0,
            parent_directories: vec!["/test".to_string()],
        })
        .collect();

    // Store in Qdrant
    vector_store
        .upsert_points(points)
        .await
        .expect("Failed to upsert points");
    println!("✓ Successfully stored vectors in Qdrant");

    // Test search
    let query_embedding = embedding_provider
        .generate_embedding("machine learning".to_string())
        .await
        .expect("Failed to generate query embedding");

    let search_results = vector_store
        .search(query_embedding, 5)
        .await
        .expect("Failed to search vectors");

    println!("✓ Search returned {} results", search_results.len());
    for (i, result) in search_results.iter().enumerate() {
        println!(
            "  Result {}: {} (score: {:.3})",
            i, result.file_path, result.score
        );
    }

    assert!(!search_results.is_empty(), "Search should return results");

    // Cleanup - delete test points
    for i in 0..2 {
        let file_path = format!("/test/document_{}.txt", i);
        vector_store
            .delete_points_by_file(&file_path)
            .await
            .expect("Failed to delete test points");
    }

    println!("✓ End-to-end test completed successfully");
}
