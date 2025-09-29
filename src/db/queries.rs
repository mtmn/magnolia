use crate::db::utils::normalize_path;
use crate::models::{DirectoryEntry, FileEntry, FileStats, SearchResult};
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn recent_dirs(db_path: &PathBuf, limit: i32) -> Result<Vec<DirectoryEntry>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT path, datetime(timestamp, 'localtime') as visited
         FROM (
             SELECT * FROM directory_history 
             ORDER BY timestamp DESC 
             LIMIT ?1
         ) 
         ORDER BY timestamp ASC",
    )?;

    let entries = stmt.query_map([limit], |row| {
        Ok(DirectoryEntry {
            path: row.get(0)?,
            timestamp: Some(row.get(1)?),
            visits: None,
        })
    })?;

    entries.collect()
}

pub fn recent_files(db_path: &PathBuf, limit: i32) -> Result<Vec<FileEntry>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT path, file_type, action, datetime(timestamp, 'localtime') as opened
         FROM (
             SELECT * FROM file_history 
             ORDER BY timestamp DESC 
             LIMIT ?1
         ) 
         ORDER BY timestamp ASC",
    )?;

    let entries = stmt.query_map([limit], |row| {
        let raw_path: String = row.get(0)?;
        Ok(FileEntry {
            path: normalize_path(&raw_path),
            file_type: row.get(1)?,
            action: row.get(2)?,
            timestamp: Some(row.get(3)?),
            opens: None,
        })
    })?;

    entries.collect()
}

pub fn popular_dirs(db_path: &PathBuf, limit: i32) -> Result<Vec<DirectoryEntry>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT path, COUNT(*) as visits, 
                datetime(MAX(timestamp), 'localtime') as last_visited
         FROM directory_history 
         GROUP BY path 
         ORDER BY visits DESC 
         LIMIT ?1",
    )?;

    let entries = stmt.query_map([limit], |row| {
        Ok(DirectoryEntry {
            path: row.get(0)?,
            visits: Some(row.get(1)?),
            timestamp: Some(row.get(2)?),
        })
    })?;

    entries.collect()
}

pub fn file_stats(db_path: &PathBuf) -> Result<Vec<FileStats>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_type, action, COUNT(*) as opens
         FROM file_history 
         GROUP BY file_type, action 
         ORDER BY opens DESC",
    )?;

    let entries = stmt.query_map([], |row| {
        Ok(FileStats {
            file_type: row.get(0)?,
            action: row.get(1)?,
            opens: row.get(2)?,
        })
    })?;

    entries.collect()
}

pub fn search_history(db_path: &PathBuf, query: &str) -> Result<SearchResult> {
    let conn = Connection::open(db_path)?;

    let mut dir_stmt = conn.prepare(
        "SELECT DISTINCT path, COUNT(*) as visits
         FROM directory_history 
         WHERE path LIKE ?1
         GROUP BY path
         ORDER BY visits DESC",
    )?;

    let dir_entries = dir_stmt.query_map([format!("%{}%", query)], |row| {
        Ok(DirectoryEntry {
            path: row.get(0)?,
            visits: Some(row.get(1)?),
            timestamp: None,
        })
    })?;

    let mut file_stmt = conn.prepare(
        "SELECT path, file_type, action, COUNT(*) as opens
         FROM file_history 
         WHERE path LIKE ?1
         GROUP BY path, file_type, action
         ORDER BY opens DESC",
    )?;

    let file_entries = file_stmt.query_map([format!("%{}%", query)], |row| {
        let raw_path: String = row.get(0)?;
        Ok(FileEntry {
            path: normalize_path(&raw_path),
            file_type: row.get(1)?,
            action: row.get(2)?,
            opens: Some(row.get(3)?),
            timestamp: None,
        })
    })?;

    Ok(SearchResult {
        directories: dir_entries.collect::<Result<Vec<_>>>()?,
        files: file_entries.collect::<Result<Vec<_>>>()?,
    })
}
