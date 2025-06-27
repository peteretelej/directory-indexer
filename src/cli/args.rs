// CLI argument parsing structures and utilities

use clap::Parser;

// This module will contain additional argument parsing utilities
// when needed for more complex CLI interactions

#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub config: Option<String>,
}
