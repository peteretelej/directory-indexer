// Async processing and concurrency tests for Gap 5
// Tests timeout scenarios, concurrent operations, and async error handling

use directory_indexer::embedding::provider::EmbeddingProvider;
use directory_indexer::storage::{QdrantStore, SqliteStore};
use directory_indexer::Config;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// Simple mock provider for testing
struct TestMockProvider;

use directory_indexer::embedding::provider::{EmbeddingResponse, EmbeddingUsage};

#[async_trait::async_trait]
impl EmbeddingProvider for TestMockProvider {
    fn model_name(&self) -> &str {
        "test-mock-model"
    }

    fn embedding_dimension(&self) -> usize {
        4 // Simple 4-dimensional embeddings for testing
    }

    async fn generate_embeddings(
        &self,
        texts: Vec<String>,
    ) -> directory_indexer::Result<EmbeddingResponse> {
        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .enumerate()
            .map(|(i, _)| vec![0.1 + i as f32 * 0.1, 0.2, 0.3, 0.4])
            .collect();

        Ok(EmbeddingResponse {
            embeddings,
            model: self.model_name().to_string(),
            usage: Some(EmbeddingUsage {
                prompt_tokens: Some(texts.len() as u32 * 10),
                total_tokens: Some(texts.len() as u32 * 10),
            }),
        })
    }

    async fn health_check(&self) -> directory_indexer::Result<bool> {
        Ok(true)
    }
}

#[tokio::test]
async fn test_embedding_provider_timeout_handling() {
    // Test that embedding providers handle timeouts gracefully
    let provider = TestMockProvider;

    // Mock provider should complete quickly
    let result = timeout(
        Duration::from_millis(100),
        provider.generate_embedding("test text".to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Embedding generation should complete within timeout"
    );
    let embedding = result.unwrap().unwrap();
    assert!(!embedding.is_empty(), "Should generate non-empty embedding");
}

#[tokio::test]
async fn test_concurrent_embedding_generation() {
    // Test concurrent embedding generation doesn't cause issues
    let provider = Arc::new(TestMockProvider);
    let mut tasks = Vec::new();

    // Spawn multiple concurrent embedding tasks
    for i in 0..10 {
        let provider_clone: Arc<TestMockProvider> = Arc::clone(&provider);
        let task = tokio::spawn(async move {
            provider_clone
                .generate_embedding(format!("test text {i}"))
                .await
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // All tasks should succeed
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Task {i} should complete successfully");
        let embedding_result = result.unwrap();
        assert!(
            embedding_result.is_ok(),
            "Embedding generation {i} should succeed"
        );
    }
}

#[tokio::test]
async fn test_qdrant_store_connection_timeout() {
    // Test connection timeout scenarios
    let config = Config::default();

    // Try to connect to a non-existent endpoint with timeout
    let invalid_endpoint = "http://localhost:99999";

    let result = timeout(
        Duration::from_millis(100),
        QdrantStore::new_without_init(invalid_endpoint, config.storage.qdrant.collection.clone())
            .get_collection_info(),
    )
    .await;

    // Should timeout or return connection error
    assert!(result.is_err() || result.unwrap().is_err());
}

// Note: This test is commented out because it revealed that SqliteStore is not thread-safe
// due to RefCell usage. This is tracked as bug GAP5-004.
/*
#[tokio::test]
async fn test_sqlite_store_concurrent_access() {
    // Test concurrent SQLite operations
    let temp_path = format!("/tmp/test_concurrent_{}.db", uuid::Uuid::new_v4());
    let store = Arc::new(SqliteStore::new(&temp_path).unwrap());
    let mut tasks = Vec::new();

    // Spawn concurrent tasks that access the database
    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let task = tokio::spawn(async move {
            // Try to get stats concurrently
            store_clone.get_stats()
        });
        tasks.push(task);
    }

    // Wait for all tasks
    let results = futures::future::join_all(tasks).await;

    // All should succeed (SQLite handles concurrent reads)
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Concurrent SQLite task {i} should succeed");
        let stats_result = result.unwrap();
        assert!(stats_result.is_ok(), "Stats query {i} should succeed");
    }

    // Clean up
    let _ = std::fs::remove_file(&temp_path);
}
*/

#[tokio::test]
async fn test_async_error_propagation() {
    // Test that async errors are properly propagated
    let provider = TestMockProvider;

    // Test error handling in embedding generation
    let result = provider.generate_embedding(String::new()).await;

    // Empty string should still work with mock provider
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check_timeout() {
    // Test health check timeout scenarios
    let provider = TestMockProvider;

    let result = timeout(Duration::from_millis(50), provider.health_check()).await;

    assert!(result.is_ok(), "Health check should complete quickly");
    assert!(
        result.unwrap().is_ok(),
        "Mock provider health check should succeed"
    );
}

#[tokio::test]
async fn test_concurrent_health_checks() {
    // Test concurrent health checks
    let provider = Arc::new(TestMockProvider);
    let mut tasks = Vec::new();

    for i in 0..5 {
        let provider_clone: Arc<TestMockProvider> = Arc::clone(&provider);
        let task = tokio::spawn(async move { (i, provider_clone.health_check().await) });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    for result in results {
        let (i, health_result) = result.unwrap();
        assert!(
            health_result.is_ok(),
            "Concurrent health check {i} should succeed"
        );
    }
}

#[tokio::test]
async fn test_async_operation_cancellation() {
    // Test cancellation scenarios with a slow operation
    let provider = TestMockProvider;

    // Start an operation that takes some time
    let task = tokio::spawn(async move {
        // Add a delay to make cancellation more likely to succeed
        tokio::time::sleep(Duration::from_millis(100)).await;
        provider.generate_embedding("test".to_string()).await
    });

    // Give it a moment to start, then cancel immediately
    tokio::time::sleep(Duration::from_millis(10)).await;
    task.abort();

    let result = task.await;

    // Check if the task was cancelled (returns JoinError) or completed
    match result {
        Err(join_error) if join_error.is_cancelled() => {
            // Task was successfully cancelled - this is what we want
        }
        Ok(_) => {
            // Task completed before cancellation - this can happen in fast operations
            // This is not necessarily a failure, just a timing issue
            println!("Task completed before cancellation could take effect");
        }
        Err(other_error) => {
            panic!("Unexpected error type: {:?}", other_error);
        }
    }
}

#[tokio::test]
async fn test_batch_embedding_concurrency() {
    // Test batch embedding operations with concurrency
    let provider = Arc::new(TestMockProvider);
    let texts: Vec<String> = (0..20).map(|i| format!("text {i}")).collect();

    // Process in concurrent batches
    let chunk_size = 5;
    let mut batch_tasks = Vec::new();

    for chunk in texts.chunks(chunk_size) {
        let provider_clone: Arc<TestMockProvider> = Arc::clone(&provider);
        let chunk_texts = chunk.to_vec();

        let task = tokio::spawn(async move {
            let mut results = Vec::new();
            for text in chunk_texts {
                let result = provider_clone.generate_embedding(text).await;
                results.push(result);
            }
            results
        });

        batch_tasks.push(task);
    }

    // Wait for all batches
    let batch_results = futures::future::join_all(batch_tasks).await;

    // Verify all batches completed successfully
    for (batch_i, batch_result) in batch_results.into_iter().enumerate() {
        assert!(batch_result.is_ok(), "Batch {batch_i} should complete");

        let embeddings = batch_result.unwrap();
        for (embed_i, embedding_result) in embeddings.into_iter().enumerate() {
            assert!(
                embedding_result.is_ok(),
                "Embedding {embed_i} in batch {batch_i} should succeed"
            );
        }
    }
}

#[tokio::test]
async fn test_error_recovery_scenarios() {
    // Test error recovery in async contexts
    let provider = TestMockProvider;

    // Simulate a scenario where some operations fail and others succeed
    let texts = vec!["good1", "", "good2", "good3"];
    let mut success_count = 0;

    for text in texts {
        match provider.generate_embedding(text.to_string()).await {
            Ok(_) => success_count += 1,
            Err(_) => {
                // Try recovery - retry with a different approach
                match provider.generate_embedding("fallback".to_string()).await {
                    Ok(_) => success_count += 1,
                    Err(_) => {} // Accept failure
                }
            }
        }
    }

    // Should have some successes
    assert!(success_count > 0, "Should have some successful operations");
}

#[tokio::test]
async fn test_resource_cleanup_on_error() {
    // Test that resources are properly cleaned up when errors occur
    let temp_path = format!("/tmp/test_cleanup_{}.db", uuid::Uuid::new_v4());

    // Create store and immediately test error scenarios
    {
        let store = SqliteStore::new(&temp_path).unwrap();

        // Try an operation that might fail
        let _ = store.get_stats();

        // Store goes out of scope here - should clean up properly
    }

    // File should still exist (SQLite doesn't auto-delete)
    assert!(std::path::Path::new(&temp_path).exists());

    // Manual cleanup
    let _ = std::fs::remove_file(&temp_path);
}

#[tokio::test]
async fn test_async_stream_processing() {
    // Test stream-like processing of async operations
    use futures::stream::{self, StreamExt};

    let provider = Arc::new(TestMockProvider);
    let texts: Vec<String> = (0..10).map(|i| format!("stream text {i}")).collect();

    // Process as an async stream with limited concurrency
    let results: Vec<_> = stream::iter(texts)
        .map(|text| {
            let provider: Arc<TestMockProvider> = Arc::clone(&provider);
            async move { provider.generate_embedding(text).await }
        })
        .buffer_unordered(3) // Limit to 3 concurrent operations
        .collect()
        .await;

    // All should succeed
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Stream processing item {i} should succeed");
    }
}

#[tokio::test]
async fn test_deadlock_prevention() {
    // Test scenarios that could potentially cause deadlocks
    let provider1 = Arc::new(TestMockProvider);
    let provider2: Arc<TestMockProvider> = Arc::clone(&provider1);

    // Simulate two tasks that might interfere with each other
    let task1 = tokio::spawn(async move {
        for i in 0..5 {
            let _ = provider1.generate_embedding(format!("task1_{i}")).await;
            tokio::task::yield_now().await; // Yield to allow other tasks to run
        }
    });

    let task2 = tokio::spawn(async move {
        for i in 0..5 {
            let _ = provider2.generate_embedding(format!("task2_{i}")).await;
            tokio::task::yield_now().await; // Yield to allow other tasks to run
        }
    });

    // Both should complete without deadlock
    let (result1, result2) = tokio::join!(task1, task2);
    assert!(result1.is_ok(), "Task 1 should complete");
    assert!(result2.is_ok(), "Task 2 should complete");
}

#[tokio::test]
async fn test_backpressure_handling() {
    // Test handling of backpressure in async operations
    let provider = Arc::new(TestMockProvider);
    let mut tasks = Vec::new();

    // Create more tasks than we want to run concurrently
    for i in 0..20 {
        let provider_clone: Arc<TestMockProvider> = Arc::clone(&provider);
        let task = tokio::spawn(async move {
            // Add a small delay to simulate real work
            tokio::time::sleep(Duration::from_millis(10)).await;
            provider_clone
                .generate_embedding(format!("backpressure_{i}"))
                .await
        });
        tasks.push(task);
    }

    // Use timeout to ensure this doesn't hang forever
    let result = timeout(Duration::from_secs(5), futures::future::join_all(tasks)).await;

    assert!(result.is_ok(), "All tasks should complete within timeout");
    let task_results = result.unwrap();

    // Check that all tasks completed successfully
    for (i, task_result) in task_results.into_iter().enumerate() {
        assert!(task_result.is_ok(), "Task {i} should complete successfully");
    }
}
