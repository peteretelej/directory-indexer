use log::{info, warn};
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
    pub file_path: PathBuf,
    pub chunk_id: usize,
    pub score: f32,
    pub content_snippet: Option<String>,
    pub parent_directories: Vec<String>,
    pub file_size: u64,
    pub modified_time: u64,
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
            results
                .into_iter()
                .filter(|result| result.file_path.starts_with(filter_dir))
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

        // TODO: Implement actual search logic
        // This would include:
        // 1. Generate embedding for the query
        // 2. Search vector store for similar chunks
        // 3. Enrich results with metadata from SQLite
        // 4. Apply directory filtering if specified
        // 5. Rank and return results

        warn!("Search not yet implemented - returning empty results");

        Ok(Vec::new())
    }

    pub async fn find_similar_files(
        &self,
        file_path: PathBuf,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        info!("Finding files similar to: {file_path:?} with limit: {limit}");

        // TODO: Implement similar file search
        // This would include:
        // 1. Get the file's chunks from the database
        // 2. Average the chunk embeddings or use the first chunk
        // 3. Search for similar chunks in vector store
        // 4. Group results by file and rank by average similarity
        // 5. Return top similar files

        warn!("Similar file search not yet implemented - returning empty results");

        Ok(Vec::new())
    }

    pub async fn get_file_content(
        &self,
        file_path: PathBuf,
        chunk_range: Option<(usize, usize)>,
    ) -> Result<String> {
        info!("Getting content for: {file_path:?} with chunks: {chunk_range:?}");

        // TODO: Implement file content retrieval
        // This would include:
        // 1. Get file record from SQLite
        // 2. If chunk_range is specified, extract only those chunks
        // 3. Otherwise, read the full file content
        // 4. Return the content

        warn!("File content retrieval not yet implemented");

        Err(IndexerError::not_found(
            "File content retrieval not implemented",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_sample_search_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                file_path: PathBuf::from("/home/user/docs/readme.md"),
                chunk_id: 0,
                score: 0.9,
                content_snippet: Some("This is a readme file".to_string()),
                parent_directories: vec!["docs".to_string()],
                file_size: 1024,
                modified_time: 1234567890,
            },
            SearchResult {
                file_path: PathBuf::from("/home/user/code/main.rs"),
                chunk_id: 1,
                score: 0.8,
                content_snippet: Some("fn main() { println!(\"Hello\"); }".to_string()),
                parent_directories: vec!["code".to_string()],
                file_size: 512,
                modified_time: 1234567891,
            },
            SearchResult {
                file_path: PathBuf::from("/home/user/docs/api.md"),
                chunk_id: 0,
                score: 0.7,
                content_snippet: Some("API documentation".to_string()),
                parent_directories: vec!["docs".to_string()],
                file_size: 2048,
                modified_time: 1234567892,
            },
            SearchResult {
                file_path: PathBuf::from("/home/user/other/test.txt"),
                chunk_id: 0,
                score: 0.5,
                content_snippet: Some("Test content".to_string()),
                parent_directories: vec!["other".to_string()],
                file_size: 256,
                modified_time: 1234567893,
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
            file_path: PathBuf::from("/test/file.txt"),
            chunk_id: 1,
            score: 0.85,
            content_snippet: Some("test content".to_string()),
            parent_directories: vec!["test".to_string()],
            file_size: 1024,
            modified_time: 1234567890,
        };

        assert_eq!(result.file_path, PathBuf::from("/test/file.txt"));
        assert_eq!(result.chunk_id, 1);
        assert_eq!(result.score, 0.85);
        assert_eq!(result.content_snippet, Some("test content".to_string()));
        assert_eq!(result.parent_directories, vec!["test".to_string()]);
        assert_eq!(result.file_size, 1024);
        assert_eq!(result.modified_time, 1234567890);
    }
}
