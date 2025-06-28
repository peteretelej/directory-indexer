use log::{info, warn};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::interval;

use crate::error::Result;

pub struct FileMonitor {
    watched_directories: Vec<PathBuf>,
    check_interval: Duration,
}

impl FileMonitor {
    pub fn new(watched_directories: Vec<PathBuf>, check_interval: Duration) -> Self {
        Self {
            watched_directories,
            check_interval,
        }
    }

    pub async fn start_monitoring<F>(&self, _on_change: F) -> Result<()>
    where
        F: Fn(FileChangeEvent) + Send + Sync + 'static,
    {
        let dir_count = self.watched_directories.len();
        info!("Starting file monitoring for {dir_count} directories");

        // TODO: Implement actual file watching
        // This could use platform-specific file watching APIs:
        // - inotify on Linux
        // - FSEvents on macOS
        // - ReadDirectoryChangesW on Windows
        //
        // For now, we'll implement a simple polling-based approach

        let mut interval = interval(self.check_interval);

        loop {
            interval.tick().await;

            // TODO: Check for file changes and call on_change callback
            // This would involve:
            // 1. Comparing current file states with stored states
            // 2. Detecting new, modified, and deleted files
            // 3. Calling the callback with appropriate events

            warn!("File monitoring not yet implemented - this is a placeholder");

            // For now, just log that we're checking
            for dir in &self.watched_directories {
                info!("Checking directory for changes: {dir:?}");
            }
        }
    }

    pub fn add_directory(&mut self, path: PathBuf) {
        if !self.watched_directories.contains(&path) {
            self.watched_directories.push(path);
        }
    }

    pub fn remove_directory(&mut self, path: &PathBuf) {
        self.watched_directories.retain(|p| p != path);
    }
}

#[derive(Debug, Clone)]
pub enum FileChangeEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

impl FileChangeEvent {
    pub fn path(&self) -> &PathBuf {
        match self {
            FileChangeEvent::Created(path) => path,
            FileChangeEvent::Modified(path) => path,
            FileChangeEvent::Deleted(path) => path,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            FileChangeEvent::Created(_) => "created",
            FileChangeEvent::Modified(_) => "modified",
            FileChangeEvent::Deleted(_) => "deleted",
        }
    }
}
