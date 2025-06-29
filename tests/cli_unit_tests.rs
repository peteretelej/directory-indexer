// CLI unit tests for argument parsing and command structure
// These tests validate CLI parsing logic without requiring external services

use clap::{CommandFactory, Parser};

// Import the main CLI structures from the binary
// We'll create a local copy to avoid dependency issues with main.rs

#[derive(Parser)]
#[command(name = "directory-indexer")]
#[command(about = "AI-powered directory indexing with semantic search")]
#[command(version)]
struct TestCli {
    #[command(subcommand)]
    command: TestCommands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    config: Option<String>,
}

#[derive(clap::Subcommand)]
enum TestCommands {
    /// Index directories for semantic search
    Index {
        /// Directory paths to index
        paths: Vec<String>,
    },
    /// Search indexed content semantically
    Search {
        /// Search query
        query: String,
        /// Optional directory to scope search
        #[arg(short, long)]
        path: Option<String>,
        /// Maximum number of results to return
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Find files similar to a given file
    Similar {
        /// Path to the reference file
        file: String,
        /// Maximum number of similar files to return
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Get file content with optional chunk selection
    Get {
        /// Path to the file
        file: String,
        /// Chunk range (e.g., "2-5")
        #[arg(long)]
        chunks: Option<String>,
    },
    /// Start MCP server
    Serve,
    /// Show indexing status and statistics
    Status {
        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_help_generation() {
        let mut command = TestCli::command();
        let help_output = command.render_help().to_string();

        assert!(help_output.contains("AI-powered directory indexing"));
        assert!(help_output.contains("Commands:"));
        assert!(help_output.contains("index"));
        assert!(help_output.contains("search"));
        assert!(help_output.contains("similar"));
        assert!(help_output.contains("get"));
        assert!(help_output.contains("serve"));
        assert!(help_output.contains("status"));
    }

    #[test]
    fn test_cli_version_option() {
        let command = TestCli::command();

        // Check that version is available via --version flag
        let help_output = command.clone().render_help().to_string();
        assert!(help_output.contains("--version"));
        assert!(help_output.contains("--help"));
    }

    #[test]
    fn test_global_args_parsing() {
        // Test verbose flag
        let cli = TestCli::try_parse_from(["directory-indexer", "-v", "serve"]).unwrap();
        assert!(cli.verbose);

        // Test config option
        let cli = TestCli::try_parse_from([
            "directory-indexer",
            "--config",
            "/path/to/config.toml",
            "serve",
        ])
        .unwrap();
        assert_eq!(cli.config, Some("/path/to/config.toml".to_string()));

        // Test both together
        let cli =
            TestCli::try_parse_from(["directory-indexer", "-v", "--config", "test.toml", "serve"])
                .unwrap();
        assert!(cli.verbose);
        assert_eq!(cli.config, Some("test.toml".to_string()));
    }

    #[test]
    fn test_index_command_parsing() {
        // Single path
        let cli = TestCli::try_parse_from(["directory-indexer", "index", "/path/to/dir"]).unwrap();

        match cli.command {
            TestCommands::Index { paths } => {
                assert_eq!(paths, vec!["/path/to/dir"]);
            }
            _ => panic!("Expected Index command"),
        }

        // Multiple paths
        let cli = TestCli::try_parse_from([
            "directory-indexer",
            "index",
            "/path/to/dir1",
            "/path/to/dir2",
            "/home/user/docs",
        ])
        .unwrap();

        match cli.command {
            TestCommands::Index { paths } => {
                assert_eq!(
                    paths,
                    vec!["/path/to/dir1", "/path/to/dir2", "/home/user/docs"]
                );
            }
            _ => panic!("Expected Index command"),
        }

        // No paths should still parse successfully (empty vector)
        let result = TestCli::try_parse_from(["directory-indexer", "index"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_command_parsing() {
        // Basic search
        let cli = TestCli::try_parse_from(["directory-indexer", "search", "test query"]).unwrap();

        match cli.command {
            TestCommands::Search { query, path, limit } => {
                assert_eq!(query, "test query");
                assert!(path.is_none());
                assert!(limit.is_none());
            }
            _ => panic!("Expected Search command"),
        }

        // Search with path and limit
        let cli = TestCli::try_parse_from([
            "directory-indexer",
            "search",
            "database timeout",
            "--path",
            "/home/user/logs",
            "--limit",
            "20",
        ])
        .unwrap();

        match cli.command {
            TestCommands::Search { query, path, limit } => {
                assert_eq!(query, "database timeout");
                assert_eq!(path, Some("/home/user/logs".to_string()));
                assert_eq!(limit, Some(20));
            }
            _ => panic!("Expected Search command"),
        }

        // Search with short flags
        let cli = TestCli::try_parse_from([
            "directory-indexer",
            "search",
            "error logs",
            "-p",
            "/var/log",
            "-l",
            "5",
        ])
        .unwrap();

        match cli.command {
            TestCommands::Search { query, path, limit } => {
                assert_eq!(query, "error logs");
                assert_eq!(path, Some("/var/log".to_string()));
                assert_eq!(limit, Some(5));
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_similar_command_parsing() {
        // Default limit
        let cli =
            TestCli::try_parse_from(["directory-indexer", "similar", "/path/to/file.md"]).unwrap();

        match cli.command {
            TestCommands::Similar { file, limit } => {
                assert_eq!(file, "/path/to/file.md");
                assert_eq!(limit, 10); // default value
            }
            _ => panic!("Expected Similar command"),
        }

        // Custom limit
        let cli = TestCli::try_parse_from([
            "directory-indexer",
            "similar",
            "/path/to/file.txt",
            "--limit",
            "25",
        ])
        .unwrap();

        match cli.command {
            TestCommands::Similar { file, limit } => {
                assert_eq!(file, "/path/to/file.txt");
                assert_eq!(limit, 25);
            }
            _ => panic!("Expected Similar command"),
        }

        // Short flag
        let cli = TestCli::try_parse_from(["directory-indexer", "similar", "test.rs", "-l", "5"])
            .unwrap();

        match cli.command {
            TestCommands::Similar { file, limit } => {
                assert_eq!(file, "test.rs");
                assert_eq!(limit, 5);
            }
            _ => panic!("Expected Similar command"),
        }
    }

    #[test]
    fn test_get_command_parsing() {
        // Without chunks
        let cli =
            TestCli::try_parse_from(["directory-indexer", "get", "/path/to/file.txt"]).unwrap();

        match cli.command {
            TestCommands::Get { file, chunks } => {
                assert_eq!(file, "/path/to/file.txt");
                assert!(chunks.is_none());
            }
            _ => panic!("Expected Get command"),
        }

        // With chunks
        let cli = TestCli::try_parse_from([
            "directory-indexer",
            "get",
            "/path/to/file.md",
            "--chunks",
            "2-5",
        ])
        .unwrap();

        match cli.command {
            TestCommands::Get { file, chunks } => {
                assert_eq!(file, "/path/to/file.md");
                assert_eq!(chunks, Some("2-5".to_string()));
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_serve_command_parsing() {
        let cli = TestCli::try_parse_from(["directory-indexer", "serve"]).unwrap();

        match cli.command {
            TestCommands::Serve => {
                // Success - no additional fields to check
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_status_command_parsing() {
        // Default format
        let cli = TestCli::try_parse_from(["directory-indexer", "status"]).unwrap();

        match cli.command {
            TestCommands::Status { format } => {
                assert_eq!(format, "text"); // default value
            }
            _ => panic!("Expected Status command"),
        }

        // JSON format
        let cli =
            TestCli::try_parse_from(["directory-indexer", "status", "--format", "json"]).unwrap();

        match cli.command {
            TestCommands::Status { format } => {
                assert_eq!(format, "json");
            }
            _ => panic!("Expected Status command"),
        }

        // Short flag
        let cli = TestCli::try_parse_from(["directory-indexer", "status", "-f", "text"]).unwrap();

        match cli.command {
            TestCommands::Status { format } => {
                assert_eq!(format, "text");
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_invalid_arguments() {
        // Unknown command
        let result = TestCli::try_parse_from(["directory-indexer", "unknown"]);
        assert!(result.is_err());

        // Invalid limit values
        let result = TestCli::try_parse_from([
            "directory-indexer",
            "search",
            "query",
            "--limit",
            "not-a-number",
        ]);
        assert!(result.is_err());

        let result =
            TestCli::try_parse_from(["directory-indexer", "similar", "file.txt", "--limit", "-5"]);
        assert!(result.is_err());

        // Missing required arguments
        let result = TestCli::try_parse_from(["directory-indexer", "search"]);
        assert!(result.is_err());

        let result = TestCli::try_parse_from(["directory-indexer", "similar"]);
        assert!(result.is_err());

        let result = TestCli::try_parse_from(["directory-indexer", "get"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_global_args_with_all_commands() {
        let commands = vec![
            vec!["index", "/path"],
            vec!["search", "query"],
            vec!["similar", "file.txt"],
            vec!["get", "file.txt"],
            vec!["serve"],
            vec!["status"],
        ];

        for cmd_args in commands {
            // Test verbose flag works with each command
            let mut args = vec!["directory-indexer", "-v"];
            args.extend(cmd_args.iter());

            let cli = TestCli::try_parse_from(args).unwrap();
            assert!(cli.verbose);

            // Test config option works with each command
            let mut args = vec!["directory-indexer", "--config", "test.toml"];
            args.extend(cmd_args.iter());

            let cli = TestCli::try_parse_from(args).unwrap();
            assert_eq!(cli.config, Some("test.toml".to_string()));
        }
    }

    #[test]
    fn test_argument_order_flexibility() {
        // Global args before command
        let cli1 = TestCli::try_parse_from([
            "directory-indexer",
            "-v",
            "--config",
            "test.toml",
            "search",
            "query",
        ])
        .unwrap();
        assert!(cli1.verbose);
        assert_eq!(cli1.config, Some("test.toml".to_string()));

        // Global args after command (should also work due to global flag)
        let cli2 = TestCli::try_parse_from([
            "directory-indexer",
            "search",
            "query",
            "-v",
            "--config",
            "test.toml",
        ])
        .unwrap();
        assert!(cli2.verbose);
        assert_eq!(cli2.config, Some("test.toml".to_string()));
    }

    #[test]
    fn test_help_subcommand_descriptions() {
        let command = TestCli::command();

        // Get subcommands and check they have descriptions
        let subcommands: Vec<_> = command.get_subcommands().collect();

        let index_cmd = subcommands
            .iter()
            .find(|cmd| cmd.get_name() == "index")
            .unwrap();
        assert!(index_cmd
            .get_about()
            .unwrap()
            .to_string()
            .contains("Index directories"));

        let search_cmd = subcommands
            .iter()
            .find(|cmd| cmd.get_name() == "search")
            .unwrap();
        assert!(search_cmd
            .get_about()
            .unwrap()
            .to_string()
            .contains("Search indexed content"));

        let similar_cmd = subcommands
            .iter()
            .find(|cmd| cmd.get_name() == "similar")
            .unwrap();
        assert!(similar_cmd
            .get_about()
            .unwrap()
            .to_string()
            .contains("Find files similar"));

        let get_cmd = subcommands
            .iter()
            .find(|cmd| cmd.get_name() == "get")
            .unwrap();
        assert!(get_cmd
            .get_about()
            .unwrap()
            .to_string()
            .contains("Get file content"));

        let serve_cmd = subcommands
            .iter()
            .find(|cmd| cmd.get_name() == "serve")
            .unwrap();
        assert!(serve_cmd
            .get_about()
            .unwrap()
            .to_string()
            .contains("Start MCP server"));

        let status_cmd = subcommands
            .iter()
            .find(|cmd| cmd.get_name() == "status")
            .unwrap();
        assert!(status_cmd
            .get_about()
            .unwrap()
            .to_string()
            .contains("Show indexing status"));
    }
}
