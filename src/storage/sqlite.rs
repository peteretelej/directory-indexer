use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

use crate::{error::Result, utils::normalize_path};

pub struct SqliteStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub size: i64,
    pub modified_time: i64,
    pub hash: String,
    pub parent_dirs: Vec<String>,
    pub chunks_json: Option<Value>,
    pub errors_json: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct DirectoryRecord {
    pub id: i64,
    pub path: String,
    pub status: String,
    pub indexed_at: i64,
}

impl SqliteStore {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let store = SqliteStore { conn };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS directories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                indexed_at INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                size INTEGER NOT NULL,
                modified_time INTEGER NOT NULL,
                hash TEXT NOT NULL,
                parent_dirs TEXT NOT NULL,
                chunks_json TEXT,
                errors_json TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_parent_dirs ON files(parent_dirs)",
            [],
        )?;

        Ok(())
    }

    pub fn add_directory(&self, path: &str) -> Result<i64> {
        let normalized_path = normalize_path(path)?;
        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO directories (path, status, indexed_at) 
             VALUES (?1, 'pending', strftime('%s', 'now'))",
        )?;

        stmt.execute([&normalized_path])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_directory_status(&self, path: &str, status: &str) -> Result<()> {
        let normalized_path = normalize_path(path)?;
        let mut stmt = self.conn.prepare(
            "UPDATE directories SET status = ?1, indexed_at = strftime('%s', 'now') WHERE path = ?2"
        )?;

        stmt.execute([status, &normalized_path])?;
        Ok(())
    }

    pub fn get_directories(&self) -> Result<Vec<DirectoryRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, status, indexed_at FROM directories ORDER BY path")?;

        let rows = stmt.query_map([], |row| {
            Ok(DirectoryRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                status: row.get(2)?,
                indexed_at: row.get(3)?,
            })
        })?;

        let mut directories = Vec::new();
        for row in rows {
            directories.push(row?);
        }

        Ok(directories)
    }

    pub fn add_file(&self, record: &FileRecord) -> Result<i64> {
        let parent_dirs_json = serde_json::to_string(&record.parent_dirs)?;
        let chunks_json = record
            .chunks_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let errors_json = record
            .errors_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO files 
             (path, size, modified_time, hash, parent_dirs, chunks_json, errors_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;

        stmt.execute(rusqlite::params![
            record.path,
            record.size,
            record.modified_time,
            record.hash,
            parent_dirs_json,
            chunks_json,
            errors_json
        ])?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_file_by_path(&self, path: &str) -> Result<Option<FileRecord>> {
        let normalized_path = normalize_path(path)?;
        let mut stmt = self.conn.prepare(
            "SELECT id, path, size, modified_time, hash, parent_dirs, chunks_json, errors_json 
             FROM files WHERE path = ?1",
        )?;

        let mut rows = stmt.query_map([&normalized_path], |row| {
            let parent_dirs_str: String = row.get(5)?;
            let parent_dirs: Vec<String> = serde_json::from_str(&parent_dirs_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let chunks_json: Option<String> = row.get(6)?;
            let chunks = chunks_json
                .filter(|s| !s.is_empty())
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            let errors_json: Option<String> = row.get(7)?;
            let errors = errors_json
                .filter(|s| !s.is_empty())
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        7,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                size: row.get(2)?,
                modified_time: row.get(3)?,
                hash: row.get(4)?,
                parent_dirs,
                chunks_json: chunks,
                errors_json: errors,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        let normalized_path = normalize_path(path)?;
        let mut stmt = self.conn.prepare("DELETE FROM files WHERE path = ?1")?;
        stmt.execute([&normalized_path])?;
        Ok(())
    }

    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        let directory_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM directories", [], |row| row.get(0))?;

        let file_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;

        let chunk_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM files WHERE chunks_json IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        Ok((directory_count, file_count, chunk_count))
    }
}
