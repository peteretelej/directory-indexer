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
