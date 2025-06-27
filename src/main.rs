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
        #[arg(short, long)]
        chunks: Option<String>,
    },
    /// Start MCP server
    Serve,
    /// Show indexing status and statistics
    Status,
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
        Commands::Search { query, path } => {
            cli::commands::search(query, path).await?;
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
        Commands::Status => {
            cli::commands::status().await?;
        }
    }

    Ok(())
}
