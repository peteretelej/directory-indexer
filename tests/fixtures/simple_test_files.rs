use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub struct SimpleTestDirectoryStructure {
    pub temp_dir: TempDir,
}

impl SimpleTestDirectoryStructure {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        Self::create_simple_test_structure(temp_dir.path());

        Self { temp_dir }
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn create_simple_test_structure(base_path: &Path) {
        // Create just 3 simple files for minimal testing

        fs::write(
            base_path.join("simple.txt"),
            "This is a simple text file for testing indexing functionality.",
        )
        .unwrap();

        fs::write(
            base_path.join("config.json"),
            r#"{"name": "test", "value": 42}"#,
        )
        .unwrap();

        fs::write(base_path.join("readme.md"), "# Test\nA minimal test file.").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_structure_creation() {
        let structure = SimpleTestDirectoryStructure::new();
        let base_path = structure.path();

        assert!(base_path.join("simple.txt").exists());
        assert!(base_path.join("config.json").exists());
        assert!(base_path.join("readme.md").exists());

        // Should only create 3 files
        let file_count = std::fs::read_dir(base_path).unwrap().count();
        assert_eq!(file_count, 3);
    }
}
