# Dependency Policy

## Philosophy

This project follows a **minimal dependency** approach. Every dependency must be justified and essential.

## Guidelines

### ✅ **When to Add Dependencies**

1. **Core Functionality**: Essential for primary features (e.g., `tokio` for async, `rusqlite` for database)
2. **Standard Ecosystem**: Well-established crates that are de facto standards (e.g., `serde`, `clap`)
3. **Safety Critical**: Prevents security vulnerabilities or memory safety issues
4. **Major Time Savings**: Implementing from scratch would take weeks and introduce bugs

### ❌ **When NOT to Add Dependencies**

1. **Convenience Only**: Can be implemented in < 50 lines of code
2. **Overlapping Functionality**: We already have a solution that works
3. **Unstable/Experimental**: Version < 1.0 or rapidly changing APIs
4. **Large Dependency Trees**: Pulls in many transitive dependencies
5. **Niche Use Cases**: Only used in one small part of the codebase

## Current Dependencies

### Essential (Core Functionality)
- `tokio` - Async runtime (required for file I/O, networking)
- `rusqlite` - SQLite database (metadata storage)
- `qdrant-client` - Vector database client (core search functionality)
- `reqwest` - HTTP client (embedding API calls)
- `serde` + `serde_json` - Serialization (config, API responses)

### CLI & User Interface
- `clap` - Command line parsing (standard in Rust ecosystem)
- `log` + `env_logger` - Logging (minimal standard approach)

### Error Handling & Quality
- `anyhow` - Error handling (error context and chaining)
- `thiserror` - Custom error types (derives Error trait)

### File Operations
- `walkdir` - Directory traversal (handles symlinks, permissions safely)
- `sha2` - File hashing (change detection)

### Configuration
- `config` - Config file parsing (supports multiple formats)
- `dirs` - OS-specific directories (cross-platform home dir, etc.)

### Development & Testing
- `tempfile` - Test fixtures (dev-dependencies only)
- `assert_cmd` - CLI testing (dev-dependencies only)
- `predicates` - Test assertions (dev-dependencies only)

## Evaluation Process

Before adding any new dependency:

1. **Check Standard Library**: Can this be done with `std`?
2. **Check Existing Dependencies**: Do we already have something that can do this?
3. **Implement Small Version**: Can we write a 20-line function instead?
4. **Cost-Benefit Analysis**: 
   - Lines of code saved vs dependency weight
   - Maintenance burden vs time savings
   - Security implications

## Examples

### ✅ **Good Dependency Additions**
```toml
# Essential for async file I/O
tokio = { version = "1.0", features = ["full"] }

# Standard for CLI parsing, saves hundreds of lines
clap = { version = "4.0", features = ["derive"] }

# Critical for security (proper HTTP client)
reqwest = { version = "0.11", features = ["json"] }
```

### ❌ **Bad Dependency Additions**
```toml
# DON'T: uuid crate for simple ID generation
uuid = "1.0"  # Can use std::collections::HashMap with incrementing IDs

# DON'T: regex for simple string operations  
regex = "1.0"  # Use string methods: contains(), starts_with(), etc.

# DON'T: fancy terminal colors for simple CLI
termcolor = "1.0"  # Use ANSI escape codes or no colors

# DON'T: complex validation library
validator = "0.16"  # Write simple validation functions
```

## Review Checklist

When reviewing PRs that add dependencies:

- [ ] Is this dependency essential for core functionality?
- [ ] Could this be implemented in <50 lines of our own code?
- [ ] Is this crate well-maintained and stable (>1.0)?
- [ ] Does this significantly improve security or performance?
- [ ] Have we considered alternatives?
- [ ] Is the dependency tree reasonable?

## Removing Dependencies

Regularly audit dependencies:
- Remove unused crates
- Consolidate overlapping functionality  
- Replace large dependencies with smaller alternatives
- Implement simple functionality in-house

The goal is a **lean, fast, secure** binary that's easy to audit and maintain.