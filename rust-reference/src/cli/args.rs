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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_common_args_default_values() {
        let args = CommonArgs {
            verbose: false,
            config: None,
        };

        assert!(!args.verbose);
        assert!(args.config.is_none());
    }

    #[test]
    fn test_common_args_with_values() {
        let args = CommonArgs {
            verbose: true,
            config: Some("/path/to/config.toml".to_string()),
        };

        assert!(args.verbose);
        assert_eq!(args.config, Some("/path/to/config.toml".to_string()));
    }

    #[test]
    fn test_common_args_debug_format() {
        let args = CommonArgs {
            verbose: true,
            config: Some("test.toml".to_string()),
        };

        let debug_output = format!("{args:?}");
        assert!(debug_output.contains("verbose: true"));
        assert!(debug_output.contains("test.toml"));
    }

    #[test]
    fn test_common_args_can_build_command() {
        // Test that the CommonArgs can be used to build a clap command
        let command = CommonArgs::command();
        assert_eq!(command.get_name(), "directory-indexer");

        // Check that the arguments are properly defined
        let args: Vec<_> = command.get_arguments().collect();
        assert!(args.iter().any(|arg| arg.get_id() == "verbose"));
        assert!(args.iter().any(|arg| arg.get_id() == "config"));
    }

    #[test]
    fn test_verbose_flag_properties() {
        let command = CommonArgs::command();
        let verbose_arg = command
            .get_arguments()
            .find(|arg| arg.get_id() == "verbose")
            .expect("verbose argument should exist");

        assert!(verbose_arg.get_short() == Some('v'));
        assert!(verbose_arg.get_long() == Some("verbose"));
        assert!(verbose_arg.is_global_set());
        assert!(verbose_arg.get_action().takes_values() == false); // It's a flag, not a value
    }

    #[test]
    fn test_config_arg_properties() {
        let command = CommonArgs::command();
        let config_arg = command
            .get_arguments()
            .find(|arg| arg.get_id() == "config")
            .expect("config argument should exist");

        assert!(config_arg.get_short() == Some('c'));
        assert!(config_arg.get_long() == Some("config"));
        assert!(config_arg.is_global_set());
        assert!(config_arg.get_action().takes_values() == true); // It takes a value
    }
}
