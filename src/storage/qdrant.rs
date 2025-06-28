use log::{debug, info, warn};
use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{IndexerError, Result};

pub struct QdrantStore {
    client: Client,
    endpoint: String,
    collection_name: String,
}

#[derive(Debug, Clone)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub file_path: String,
    pub chunk_id: usize,
    pub parent_directories: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub chunk_id: usize,
    pub score: f32,
    pub parent_directories: Vec<String>,
}

impl QdrantStore {
    pub async fn new(endpoint: &str, collection_name: String) -> Result<Self> {
        Self::new_with_api_key(endpoint, collection_name, None).await
    }

    pub async fn new_with_api_key(
        endpoint: &str,
        collection_name: String,
        _api_key: Option<String>,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                IndexerError::vector_store(format!("Failed to create HTTP client: {e}"))
            })?;

        let store = QdrantStore {
            client,
            endpoint: endpoint.to_string(),
            collection_name,
        };

        store.ensure_collection_exists().await?;
        Ok(store)
    }

    pub fn new_without_init(endpoint: &str, collection_name: String) -> Self {
        QdrantStore {
            client: Client::new(),
            endpoint: endpoint.to_string(),
            collection_name,
        }
    }

    async fn ensure_collection_exists(&self) -> Result<()> {
        // Check if collection exists
        let url = format!(
            "{}/collections/{}/exists",
            self.endpoint, self.collection_name
        );
        let response = self.client.get(&url).send().await.map_err(|e| {
            IndexerError::vector_store(format!("Failed to check collection existence: {e}"))
        })?;

        let response_text = response
            .text()
            .await
            .map_err(|e| IndexerError::vector_store(format!("Failed to get response text: {e}")))?;

        debug!("Collection exists response: {response_text}");

        let exists_response: Value = serde_json::from_str(&response_text).map_err(|e| {
            IndexerError::vector_store(format!(
                "Failed to parse collection exists response: {e} - Response was: {response_text}"
            ))
        })?;

        let collection_exists = exists_response
            .get("result")
            .and_then(|r| r.get("exists"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        if !collection_exists {
            info!("Creating Qdrant collection: {}", self.collection_name);
            self.create_collection().await?;
        } else {
            debug!(
                "Qdrant collection '{}' already exists",
                self.collection_name
            );
        }

        Ok(())
    }

    async fn create_collection(&self) -> Result<()> {
        let url = format!("{}/collections/{}", self.endpoint, self.collection_name);

        let create_payload = json!({
            "vectors": {
                "size": 768, // Default dimension, will be updated based on actual embeddings
                "distance": "Cosine"
            },
            "shard_number": 1,
            "replication_factor": 1
        });

        let response = self
            .client
            .put(&url)
            .json(&create_payload)
            .send()
            .await
            .map_err(|e| IndexerError::vector_store(format!("Failed to create collection: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(IndexerError::vector_store(format!(
                "Failed to create collection: HTTP {status} - {error_text}"
            )));
        }

        info!(
            "Successfully created Qdrant collection: {}",
            self.collection_name
        );
        Ok(())
    }

    pub async fn upsert_points(&self, points: Vec<VectorPoint>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        debug!("Upserting {} points to Qdrant", points.len());

        let qdrant_points: Vec<Value> = points
            .into_iter()
            .map(|point| {
                json!({
                    "id": point.id,
                    "vector": point.vector,
                    "payload": {
                        "file_path": point.file_path,
                        "chunk_id": point.chunk_id,
                        "parent_directories": point.parent_directories
                    }
                })
            })
            .collect();

        let upsert_payload = json!({
            "points": qdrant_points
        });

        let url = format!(
            "{}/collections/{}/points",
            self.endpoint, self.collection_name
        );
        let response = self
            .client
            .put(&url)
            .json(&upsert_payload)
            .send()
            .await
            .map_err(|e| IndexerError::vector_store(format!("Failed to upsert points: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(IndexerError::vector_store(format!(
                "Failed to upsert points: HTTP {status} - {error_text}"
            )));
        }

        debug!("Successfully upserted points to Qdrant");
        Ok(())
    }

    pub async fn search(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        debug!(
            "Searching Qdrant with vector dimension: {}, limit: {limit}",
            query_vector.len()
        );

        let search_payload = json!({
            "vector": query_vector,
            "limit": limit,
            "with_payload": true,
            "score_threshold": 0.0
        });

        let url = format!(
            "{}/collections/{}/points/search",
            self.endpoint, self.collection_name
        );
        let response = self
            .client
            .post(&url)
            .json(&search_payload)
            .send()
            .await
            .map_err(|e| IndexerError::vector_store(format!("Failed to search points: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(IndexerError::vector_store(format!(
                "Failed to search points: HTTP {status} - {error_text}"
            )));
        }

        let search_response: Value = response.json().await.map_err(|e| {
            IndexerError::vector_store(format!("Failed to parse search response: {e}"))
        })?;

        let results: Vec<SearchResult> = search_response
            .get("result")
            .and_then(|r| r.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|point| {
                let payload = point.get("payload")?;
                let file_path = payload.get("file_path")?.as_str()?.to_string();
                let chunk_id = payload.get("chunk_id")?.as_u64()? as usize;
                let parent_directories = payload
                    .get("parent_directories")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let score = point.get("score")?.as_f64()? as f32;

                Some(SearchResult {
                    file_path,
                    chunk_id,
                    score,
                    parent_directories,
                })
            })
            .collect();

        debug!("Found {} search results", results.len());
        Ok(results)
    }

    pub async fn delete_points_by_file(&self, file_path: &str) -> Result<()> {
        debug!("Deleting points for file: {file_path}");

        let delete_payload = json!({
            "filter": {
                "must": [
                    {
                        "key": "file_path",
                        "match": {
                            "value": file_path
                        }
                    }
                ]
            }
        });

        let url = format!(
            "{}/collections/{}/points/delete",
            self.endpoint, self.collection_name
        );
        let response = self
            .client
            .post(&url)
            .json(&delete_payload)
            .send()
            .await
            .map_err(|e| IndexerError::vector_store(format!("Failed to delete points: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(IndexerError::vector_store(format!(
                "Failed to delete points: HTTP {status} - {error_text}"
            )));
        }

        debug!("Successfully deleted points for file: {file_path}");
        Ok(())
    }

    pub async fn get_collection_info(&self) -> Result<CollectionInfo> {
        let url = format!("{}/collections/{}", self.endpoint, self.collection_name);
        let response = self.client.get(&url).send().await.map_err(|e| {
            IndexerError::vector_store(format!("Failed to get collection info: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(IndexerError::vector_store(format!(
                "Failed to get collection info: HTTP {status} - {error_text}"
            )));
        }

        let collection_info: Value = response.json().await.map_err(|e| {
            IndexerError::vector_store(format!("Failed to parse collection info: {e}"))
        })?;

        let points_count = collection_info
            .get("result")
            .and_then(|r| r.get("points_count"))
            .and_then(|p| p.as_u64())
            .unwrap_or(0);

        let indexed_vectors_count = collection_info
            .get("result")
            .and_then(|r| r.get("indexed_vectors_count"))
            .and_then(|p| p.as_u64())
            .unwrap_or(0);

        Ok(CollectionInfo {
            points_count,
            indexed_vectors_count,
        })
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/", self.endpoint);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Qdrant health check failed: {e}");
                Ok(false)
            }
        }
    }

    pub async fn delete_collection(&self) -> Result<()> {
        let url = format!("{}/collections/{}", self.endpoint, self.collection_name);
        let response =
            self.client.delete(&url).send().await.map_err(|e| {
                IndexerError::vector_store(format!("Failed to delete collection: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            // Don't error if collection doesn't exist (404)
            if status != reqwest::StatusCode::NOT_FOUND {
                return Err(IndexerError::vector_store(format!(
                    "Failed to delete collection: HTTP {status} - {error_text}"
                )));
            }
        }

        debug!("Successfully deleted collection: {}", self.collection_name);
        Ok(())
    }
}

#[derive(Debug)]
pub struct CollectionInfo {
    pub points_count: u64,
    pub indexed_vectors_count: u64,
}
