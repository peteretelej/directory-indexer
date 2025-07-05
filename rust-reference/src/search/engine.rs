use log::info;
use std::path::PathBuf;

use crate::{
    embedding::EmbeddingProvider,
    error::{IndexerError, Result},
    storage::{QdrantStore, SqliteStore},
};

pub struct SearchEngine {
    #[allow(dead_code)]
    sqlite_store: SqliteStore,
    #[allow(dead_code)]
    vector_store: QdrantStore,
    #[allow(dead_code)]
    embedding_provider: Box<dyn EmbeddingProvider>,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub directory_filter: Option<PathBuf>,
    pub limit: usize,
    pub similarity_threshold: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub chunk_id: usize,
    pub score: f32,
    pub parent_directories: Vec<String>,
}

impl SearchEngine {
    pub fn new(
        sqlite_store: SqliteStore,
        vector_store: QdrantStore,
        embedding_provider: Box<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            sqlite_store,
            vector_store,
            embedding_provider,
        }
    }

    pub fn validate_query(&self, query: &SearchQuery) -> Result<()> {
        Self::validate_query_static(query)
    }

    pub fn filter_results_by_directory(
        &self,
        results: Vec<SearchResult>,
        directory_filter: &Option<PathBuf>,
    ) -> Vec<SearchResult> {
        Self::filter_results_by_directory_static(results, directory_filter)
    }

    pub fn apply_similarity_threshold(
        &self,
        results: Vec<SearchResult>,
        threshold: Option<f32>,
    ) -> Vec<SearchResult> {
        Self::apply_similarity_threshold_static(results, threshold)
    }

    pub fn rank_results(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        Self::rank_results_static(results)
    }

    pub fn limit_results(&self, results: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
        Self::limit_results_static(results, limit)
    }

    // Static versions for easier unit testing
    pub fn validate_query_static(query: &SearchQuery) -> Result<()> {
        if query.text.trim().is_empty() {
            return Err(IndexerError::invalid_input("Search query cannot be empty"));
        }

        if query.limit == 0 {
            return Err(IndexerError::invalid_input(
                "Search limit must be greater than 0",
            ));
        }

        if let Some(threshold) = query.similarity_threshold {
            if !(0.0..=1.0).contains(&threshold) {
                return Err(IndexerError::invalid_input(
                    "Similarity threshold must be between 0.0 and 1.0",
                ));
            }
        }

        if let Some(ref dir_filter) = query.directory_filter {
            if !dir_filter.is_dir() && !dir_filter.exists() {
                return Err(IndexerError::invalid_input(
                    "Directory filter must be a valid directory path",
                ));
            }
        }

        Ok(())
    }

    pub fn filter_results_by_directory_static(
        results: Vec<SearchResult>,
        directory_filter: &Option<PathBuf>,
    ) -> Vec<SearchResult> {
        if let Some(filter_dir) = directory_filter {
            let filter_str = filter_dir.to_string_lossy();
            results
                .into_iter()
                .filter(|result| result.file_path.starts_with(filter_str.as_ref()))
                .collect()
        } else {
            results
        }
    }

    pub fn apply_similarity_threshold_static(
        results: Vec<SearchResult>,
        threshold: Option<f32>,
    ) -> Vec<SearchResult> {
        if let Some(min_score) = threshold {
            results
                .into_iter()
                .filter(|result| result.score >= min_score)
                .collect()
        } else {
            results
        }
    }

    pub fn rank_results_static(mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn limit_results_static(results: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
        results.into_iter().take(limit).collect()
    }

    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let text = &query.text;
        let limit = query.limit;
        info!("Searching for: '{text}' with limit: {limit}");

        // Validate query
        self.validate_query(&query)?;

        // Generate embedding for the query
        let query_embedding = self
            .embedding_provider
            .generate_embedding(query.text.clone())
            .await?;

        // Perform vector search
        let search_results = self.vector_store.search(query_embedding, limit).await?;

        // Apply directory filtering if specified
        let filtered_results =
            self.filter_results_by_directory(search_results, &query.directory_filter);

        // Apply similarity threshold if specified
        let threshold_results =
            self.apply_similarity_threshold(filtered_results, query.similarity_threshold);

        // Rank results
        let ranked_results = self.rank_results(threshold_results);

        // Limit results
        let final_results = self.limit_results(ranked_results, limit);

        Ok(final_results)
    }

    pub async fn find_similar_files(
        &self,
        file_path: PathBuf,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        info!("Finding files similar to: {file_path:?} with limit: {limit}");

        if !file_path.exists() {
            return Err(IndexerError::not_found(format!(
                "File not found: {}",
                file_path.display()
            )));
        }
        if !file_path.is_file() {
            return Err(IndexerError::invalid_input(format!(
                "Path is not a file: {}",
                file_path.display()
            )));
        }

        // Try to get file from database to retrieve chunks
        let normalized_path = crate::utils::normalize_path(&file_path)?;
        let file_record = self.sqlite_store.get_file_by_path(&normalized_path)?;

        // Generate embedding for the file
        let file_embedding = if let Some(file_record) = file_record {
            // Parse chunks JSON to get file chunks
            let chunks = match file_record.chunks_json {
                Some(chunks_json) => {
                    serde_json::from_value::<Vec<String>>(chunks_json).map_err(|e| {
                        IndexerError::file_processing(format!("Failed to parse chunks: {e}"))
                    })?
                }
                None => {
                    return Err(IndexerError::not_found(format!(
                        "No chunks found for file: {}",
                        file_path.display()
                    )));
                }
            };

            if chunks.is_empty() {
                return Err(IndexerError::not_found(format!(
                    "No chunks found for file: {}",
                    file_path.display()
                )));
            }

            // Use the first chunk as representative of the file
            let representative_chunk = &chunks[0];
            self.embedding_provider
                .generate_embedding(representative_chunk.clone())
                .await?
        } else {
            // File not indexed, read from filesystem and generate embedding
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {e}")))?;

            // Use first 512 chars as representative content
            let representative_content = if content.len() > 512 {
                &content[..512]
            } else {
                &content
            };

            self.embedding_provider
                .generate_embedding(representative_content.to_string())
                .await?
        };

        // Search for similar chunks
        let search_results = self.vector_store.search(file_embedding, limit + 5).await?;

        // Filter out results from the same file and group by file path
        let mut file_scores: std::collections::HashMap<String, (f32, usize)> =
            std::collections::HashMap::new();
        let file_path_str = file_path.to_string_lossy().to_string();

        for result in search_results {
            // Skip if it's the same file
            if result.file_path == file_path_str {
                continue;
            }

            // Keep track of the best score for each file
            let entry = file_scores
                .entry(result.file_path.clone())
                .or_insert((0.0, 0));
            if result.score > entry.0 {
                entry.0 = result.score;
                entry.1 = result.chunk_id;
            }
        }

        // Sort by score and take top results, convert to SearchResult
        let mut similar_files: Vec<_> = file_scores.into_iter().collect();
        similar_files.sort_by(|a, b| {
            b.1 .0
                .partial_cmp(&a.1 .0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        similar_files.truncate(limit);

        let results: Vec<SearchResult> = similar_files
            .into_iter()
            .map(|(file_path, (score, chunk_id))| SearchResult {
                file_path,
                chunk_id,
                score,
                parent_directories: vec![], // Could be populated if needed
            })
            .collect();

        Ok(results)
    }

    pub async fn get_file_content(
        &self,
        file_path: PathBuf,
        chunk_range: Option<(usize, usize)>,
    ) -> Result<String> {
        info!("Getting content for: {file_path:?} with chunks: {chunk_range:?}");

        if !file_path.exists() {
            return Err(IndexerError::not_found(format!(
                "File not found: {}",
                file_path.display()
            )));
        }
        if !file_path.is_file() {
            return Err(IndexerError::invalid_input(format!(
                "Path is not a file: {}",
                file_path.display()
            )));
        }

        // Try to get file from database
        let normalized_path = crate::utils::normalize_path(&file_path)?;
        let file_record = self.sqlite_store.get_file_by_path(&normalized_path)?;

        // If chunks are stored in database, use those; otherwise read from file system
        let content = if let Some(file_record) = file_record {
            if let Some(chunks_json) = file_record.chunks_json {
                let chunks = serde_json::from_value::<Vec<String>>(chunks_json).map_err(|e| {
                    IndexerError::file_processing(format!("Failed to parse chunks: {e}"))
                })?;

                if let Some((start, end)) = chunk_range {
                    // Return specific chunk range (1-indexed to 0-indexed)
                    let start_idx = start.saturating_sub(1);
                    let end_idx = end.min(chunks.len());

                    if start_idx >= chunks.len() {
                        return Err(IndexerError::invalid_input(format!(
                            "Chunk range {start}-{end} exceeds available chunks ({})",
                            chunks.len()
                        )));
                    }

                    chunks[start_idx..end_idx].join("\n")
                } else {
                    // Return all chunks
                    chunks.join("\n")
                }
            } else {
                // File indexed but no chunks stored, read from filesystem
                let content = std::fs::read_to_string(&file_path).map_err(|e| {
                    IndexerError::file_processing(format!("Failed to read file: {e}"))
                })?;

                if let Some((start, end)) = chunk_range {
                    // Split content into chunks on-the-fly for files without stored chunks
                    let lines: Vec<&str> = content.lines().collect();
                    let lines_per_chunk = lines.len().div_ceil(10); // Approximate 10 chunks
                    let total_chunks = lines.len().div_ceil(lines_per_chunk);

                    if start > total_chunks || start == 0 {
                        return Err(IndexerError::invalid_input(format!(
                            "Chunk {start} is out of range. File has {total_chunks} estimated chunks"
                        )));
                    }

                    let start_line = (start - 1) * lines_per_chunk;
                    let end_line = (end * lines_per_chunk).min(lines.len());

                    lines[start_line..end_line].join("\n")
                } else {
                    content
                }
            }
        } else {
            // File not indexed, read directly from file system
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| IndexerError::file_processing(format!("Failed to read file: {e}")))?;

            if let Some((start, end)) = chunk_range {
                // Split content into chunks on-the-fly for unindexed files
                let lines: Vec<&str> = content.lines().collect();
                let lines_per_chunk = lines.len().div_ceil(10); // Approximate 10 chunks
                let total_chunks = lines.len().div_ceil(lines_per_chunk);

                if start > total_chunks || start == 0 {
                    return Err(IndexerError::invalid_input(format!(
                        "Chunk {start} is out of range. File has {total_chunks} estimated chunks"
                    )));
                }

                let start_line = (start - 1) * lines_per_chunk;
                let end_line = (end * lines_per_chunk).min(lines.len());

                lines[start_line..end_line].join("\n")
            } else {
                content
            }
        };

        Ok(content)
    }
}

pub async fn create_search_engine() -> Result<SearchEngine> {
    let config = crate::Config::load()?;
    crate::environment::validate_environment(&config).await?;

    let sqlite_store = crate::storage::SqliteStore::new(&config.storage.sqlite_path)?;
    let vector_store = crate::storage::QdrantStore::new(
        &config.storage.qdrant.endpoint,
        config.storage.qdrant.collection.clone(),
    )
    .await?;
    let embedding_provider = crate::embedding::create_embedding_provider(&config.embedding)?;

    Ok(SearchEngine::new(
        sqlite_store,
        vector_store,
        embedding_provider,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_sample_search_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                file_path: "/home/user/docs/readme.md".to_string(),
                chunk_id: 0,
                score: 0.9,
                parent_directories: vec!["docs".to_string()],
            },
            SearchResult {
                file_path: "/home/user/code/main.rs".to_string(),
                chunk_id: 1,
                score: 0.8,
                parent_directories: vec!["code".to_string()],
            },
            SearchResult {
                file_path: "/home/user/docs/api.md".to_string(),
                chunk_id: 0,
                score: 0.7,
                parent_directories: vec!["docs".to_string()],
            },
            SearchResult {
                file_path: "/home/user/other/test.txt".to_string(),
                chunk_id: 0,
                score: 0.5,
                parent_directories: vec!["other".to_string()],
            },
        ]
    }

    #[test]
    fn test_validate_query_success() {
        let valid_query = SearchQuery {
            text: "test search".to_string(),
            directory_filter: None,
            limit: 10,
            similarity_threshold: Some(0.5),
        };

        assert!(SearchEngine::validate_query_static(&valid_query).is_ok());
    }

    #[test]
    fn test_validate_query_empty_text() {
        let invalid_query = SearchQuery {
            text: "".to_string(),
            directory_filter: None,
            limit: 10,
            similarity_threshold: None,
        };

        let result = SearchEngine::validate_query_static(&invalid_query);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_query_whitespace_only_text() {
        let invalid_query = SearchQuery {
            text: "   \t\n  ".to_string(),
            directory_filter: None,
            limit: 10,
            similarity_threshold: None,
        };

        let result = SearchEngine::validate_query_static(&invalid_query);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_query_zero_limit() {
        let invalid_query = SearchQuery {
            text: "test".to_string(),
            directory_filter: None,
            limit: 0,
            similarity_threshold: None,
        };

        let result = SearchEngine::validate_query_static(&invalid_query);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than 0"));
    }

    #[test]
    fn test_validate_query_invalid_similarity_threshold() {
        let invalid_queries = vec![
            SearchQuery {
                text: "test".to_string(),
                directory_filter: None,
                limit: 10,
                similarity_threshold: Some(-0.1),
            },
            SearchQuery {
                text: "test".to_string(),
                directory_filter: None,
                limit: 10,
                similarity_threshold: Some(1.1),
            },
        ];

        for query in invalid_queries {
            let result = SearchEngine::validate_query_static(&query);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("between 0.0 and 1.0"));
        }
    }

    #[test]
    fn test_validate_query_valid_similarity_threshold() {
        let valid_thresholds = vec![0.0, 0.5, 1.0];

        for threshold in valid_thresholds {
            let query = SearchQuery {
                text: "test".to_string(),
                directory_filter: None,
                limit: 10,
                similarity_threshold: Some(threshold),
            };

            assert!(SearchEngine::validate_query_static(&query).is_ok());
        }
    }

    #[test]
    fn test_filter_results_by_directory_with_filter() {
        let results = create_sample_search_results();

        let filter_dir = Some(PathBuf::from("/home/user/docs"));
        let filtered = SearchEngine::filter_results_by_directory_static(results, &filter_dir);

        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|r| r.file_path.starts_with("/home/user/docs")));
    }

    #[test]
    fn test_filter_results_by_directory_no_filter() {
        let results = create_sample_search_results();
        let original_count = results.len();

        let filtered = SearchEngine::filter_results_by_directory_static(results, &None);

        assert_eq!(filtered.len(), original_count);
    }

    #[test]
    fn test_filter_results_by_directory_no_matches() {
        let results = create_sample_search_results();

        let filter_dir = Some(PathBuf::from("/nonexistent/path"));
        let filtered = SearchEngine::filter_results_by_directory_static(results, &filter_dir);

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_apply_similarity_threshold_with_threshold() {
        let results = create_sample_search_results();

        let threshold = Some(0.75);
        let filtered = SearchEngine::apply_similarity_threshold_static(results, threshold);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.score >= 0.75));
    }

    #[test]
    fn test_apply_similarity_threshold_no_threshold() {
        let results = create_sample_search_results();
        let original_count = results.len();

        let filtered = SearchEngine::apply_similarity_threshold_static(results, None);

        assert_eq!(filtered.len(), original_count);
    }

    #[test]
    fn test_apply_similarity_threshold_no_matches() {
        let results = create_sample_search_results();

        let threshold = Some(0.95);
        let filtered = SearchEngine::apply_similarity_threshold_static(results, threshold);

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_rank_results() {
        let results = create_sample_search_results();

        let ranked = SearchEngine::rank_results_static(results);

        assert_eq!(ranked.len(), 4);
        assert_eq!(ranked[0].score, 0.9);
        assert_eq!(ranked[1].score, 0.8);
        assert_eq!(ranked[2].score, 0.7);
        assert_eq!(ranked[3].score, 0.5);

        // Verify it's sorted in descending order
        for i in 1..ranked.len() {
            assert!(ranked[i - 1].score >= ranked[i].score);
        }
    }

    #[test]
    fn test_rank_results_empty() {
        let ranked = SearchEngine::rank_results_static(vec![]);

        assert_eq!(ranked.len(), 0);
    }

    #[test]
    fn test_limit_results() {
        let results = create_sample_search_results();

        let limited = SearchEngine::limit_results_static(results, 2);

        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_limit_results_larger_than_available() {
        let results = create_sample_search_results();
        let original_count = results.len();

        let limited = SearchEngine::limit_results_static(results, 10);

        assert_eq!(limited.len(), original_count);
    }

    #[test]
    fn test_limit_results_zero() {
        let results = create_sample_search_results();

        let limited = SearchEngine::limit_results_static(results, 0);

        assert_eq!(limited.len(), 0);
    }

    // Integration tests for full search functionality will be in tests/search_integration_tests.rs

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery {
            text: "test query".to_string(),
            directory_filter: Some(PathBuf::from("/test/dir")),
            limit: 5,
            similarity_threshold: Some(0.8),
        };

        assert_eq!(query.text, "test query");
        assert_eq!(query.directory_filter, Some(PathBuf::from("/test/dir")));
        assert_eq!(query.limit, 5);
        assert_eq!(query.similarity_threshold, Some(0.8));
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            file_path: "/test/file.txt".to_string(),
            chunk_id: 1,
            score: 0.85,
            parent_directories: vec!["test".to_string()],
        };

        assert_eq!(result.file_path, "/test/file.txt");
        assert_eq!(result.chunk_id, 1);
        assert_eq!(result.score, 0.85);
        assert_eq!(result.parent_directories, vec!["test".to_string()]);
    }
}
