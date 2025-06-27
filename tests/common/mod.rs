use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub test_files: Vec<PathBuf>,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let test_files = Self::create_test_files(&temp_dir);
        
        Self {
            temp_dir,
            test_files,
        }
    }

    fn create_test_files(temp_dir: &TempDir) -> Vec<PathBuf> {
        let base_path = temp_dir.path();
        let mut files = Vec::new();

        // Create various test files
        let files_to_create = vec![
            ("test.md", "# Test Document\n\nThis is a test markdown file with some content."),
            ("config.json", r#"{"key": "value", "number": 42}"#),
            ("script.py", "def hello_world():\n    print('Hello, world!')"),
            ("data.csv", "name,age,city\nJohn,25,NYC\nJane,30,LA"),
            ("readme.txt", "This is a simple text file.\nIt has multiple lines."),
        ];

        for (filename, content) in files_to_create {
            let file_path = base_path.join(filename);
            fs::write(&file_path, content).unwrap();
            files.push(file_path);
        }

        // Create a subdirectory with files
        let subdir = base_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        let subfile = subdir.join("nested.md");
        fs::write(&subfile, "# Nested File\n\nThis file is in a subdirectory.").unwrap();
        files.push(subfile);

        files
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

pub fn setup_test_config() -> directory_indexer::Config {
    use directory_indexer::config::{Config, StorageConfig, QdrantConfig, EmbeddingConfig, IndexingConfig, MonitoringConfig};
    use std::path::PathBuf;

    Config {
        storage: StorageConfig {
            sqlite_path: PathBuf::from(":memory:"), // Use in-memory SQLite for tests
            qdrant: QdrantConfig {
                endpoint: std::env::var("QDRANT_URL")
                    .unwrap_or_else(|_| "http://localhost:6333".to_string()),
                collection: "test-collection".to_string(),
            },
        },
        embedding: EmbeddingConfig {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
        },
        indexing: IndexingConfig {
            chunk_size: 128, // Smaller chunks for testing
            overlap: 20,
            max_file_size: 1024 * 1024, // 1MB
            ignore_patterns: vec![".git".to_string()],
            concurrency: 1, // Single-threaded for predictable tests
        },
        monitoring: MonitoringConfig {
            file_watching: false,
            batch_size: 10,
        },
    }
}