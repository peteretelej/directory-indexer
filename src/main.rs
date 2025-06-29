use clap::{Parser, Subcommand};
use log::info;

// Re-export the main library functionality
use directory_indexer::*;

#[derive(Parser)]
#[command(name = "directory-indexer")]
#[command(about = "AI-powered directory indexing with semantic search")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
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

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", log_level);
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();

    info!("Starting directory-indexer");

    match cli.command {
        Commands::Index { paths } => {
            cli::commands::index(paths).await?;
        }
        Commands::Search { query, path, limit } => {
            cli::commands::search(query, path, limit).await?;
        }
        Commands::Similar { file, limit } => {
            cli::commands::similar(file, limit).await?;
        }
        Commands::Get { file, chunks } => {
            cli::commands::get(file, chunks).await?;
        }
        Commands::Serve => {
            cli::commands::serve().await?;
        }
        Commands::Status { format } => {
            cli::commands::status(format).await?;
        }
    }

    Ok(())
}

// Helper functions that can be tested separately from main
pub fn setup_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", log_level);
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();
}

pub fn get_log_level(verbose: bool) -> &'static str {
    if verbose {
        "debug"
    } else {
        "info"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_log_level() {
        assert_eq!(get_log_level(true), "debug");
        assert_eq!(get_log_level(false), "info");
    }

    #[test]
    fn test_cli_structure() {
        // Test that the CLI can be created and has expected structure
        let cli = Cli::parse_from(["directory-indexer", "serve"]);
        assert!(!cli.verbose); // default value
        assert!(cli.config.is_none()); // default value

        // Test verbose flag
        let cli = Cli::parse_from(["directory-indexer", "-v", "serve"]);
        assert!(cli.verbose);

        // Test config option
        let cli = Cli::parse_from(["directory-indexer", "--config", "test.toml", "serve"]);
        assert_eq!(cli.config, Some("test.toml".to_string()));
    }

    #[test]
    fn test_commands_enum_parsing() {
        // Test all command variants can be parsed
        let test_cases = vec![
            (vec!["directory-indexer", "index", "/path"], "index"),
            (vec!["directory-indexer", "search", "query"], "search"),
            (vec!["directory-indexer", "similar", "file.txt"], "similar"),
            (vec!["directory-indexer", "get", "file.txt"], "get"),
            (vec!["directory-indexer", "serve"], "serve"),
            (vec!["directory-indexer", "status"], "status"),
        ];

        for (args, expected_cmd) in test_cases {
            let cli = Cli::parse_from(args);
            match (&cli.command, expected_cmd) {
                (Commands::Index { .. }, "index") => {}
                (Commands::Search { .. }, "search") => {}
                (Commands::Similar { .. }, "similar") => {}
                (Commands::Get { .. }, "get") => {}
                (Commands::Serve, "serve") => {}
                (Commands::Status { .. }, "status") => {}
                _ => panic!("Unexpected command type for {expected_cmd}"),
            }
        }
    }

    #[test]
    fn test_command_with_global_args() {
        let cli = Cli::parse_from([
            "directory-indexer",
            "-v",
            "--config",
            "/path/to/config.toml",
            "search",
            "test query",
            "--limit",
            "5",
        ]);

        assert!(cli.verbose);
        assert_eq!(cli.config, Some("/path/to/config.toml".to_string()));

        match cli.command {
            Commands::Search { query, limit, .. } => {
                assert_eq!(query, "test query");
                assert_eq!(limit, Some(5));
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_help_content() {
        use clap::CommandFactory;

        let mut command = Cli::command();
        let help_output = command.render_help().to_string();

        // Check that help contains expected content
        assert!(help_output.contains("AI-powered directory indexing"));
        assert!(help_output.contains("Usage:"));
        assert!(help_output.contains("Commands:"));
        assert!(help_output.contains("Options:"));
    }
}
