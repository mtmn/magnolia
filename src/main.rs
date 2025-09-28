use rusqlite::{Connection, Result};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use colored_json::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryEntry {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visits: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    path: String,
    file_type: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opens: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileStats {
    file_type: String,
    action: String,
    opens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    directories: Vec<DirectoryEntry>,
    files: Vec<FileEntry>,
}

fn get_default_db_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".fzf.db")
}

fn recent_dirs(db_path: &PathBuf, limit: i32) -> Result<Vec<DirectoryEntry>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT path, datetime(timestamp, 'localtime') as visited
         FROM (
             SELECT * FROM directory_history 
             ORDER BY timestamp DESC 
             LIMIT ?1
         ) 
         ORDER BY timestamp ASC"
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

fn recent_files(db_path: &PathBuf, limit: i32) -> Result<Vec<FileEntry>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT path, file_type, action, datetime(timestamp, 'localtime') as opened
         FROM (
             SELECT * FROM file_history 
             ORDER BY timestamp DESC 
             LIMIT ?1
         ) 
         ORDER BY timestamp ASC"
    )?;
    
    let entries = stmt.query_map([limit], |row| {
        Ok(FileEntry {
            path: row.get(0)?,
            file_type: row.get(1)?,
            action: row.get(2)?,
            timestamp: Some(row.get(3)?),
            opens: None,
        })
    })?;
    
    entries.collect()
}

fn popular_dirs(db_path: &PathBuf, limit: i32) -> Result<Vec<DirectoryEntry>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT path, COUNT(*) as visits, 
                datetime(MAX(timestamp), 'localtime') as last_visited
         FROM directory_history 
         GROUP BY path 
         ORDER BY visits DESC 
         LIMIT ?1"
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

fn file_stats(db_path: &PathBuf) -> Result<Vec<FileStats>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_type, action, COUNT(*) as opens
         FROM file_history 
         GROUP BY file_type, action 
         ORDER BY opens DESC"
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

fn search_history(db_path: &PathBuf, query: &str) -> Result<SearchResult> {
    let conn = Connection::open(db_path)?;
    
    // Search directories
    let mut dir_stmt = conn.prepare(
        "SELECT DISTINCT path, COUNT(*) as visits
         FROM directory_history 
         WHERE path LIKE ?1
         GROUP BY path
         ORDER BY visits DESC"
    )?;
    
    let dir_entries = dir_stmt.query_map([format!("%{}%", query)], |row| {
        Ok(DirectoryEntry {
            path: row.get(0)?,
            visits: Some(row.get(1)?),
            timestamp: None,
        })
    })?;
    
    // Search files
    let mut file_stmt = conn.prepare(
        "SELECT path, file_type, action, COUNT(*) as opens
         FROM file_history 
         WHERE path LIKE ?1
         GROUP BY path, file_type, action
         ORDER BY opens DESC"
    )?;
    
    let file_entries = file_stmt.query_map([format!("%{}%", query)], |row| {
        Ok(FileEntry {
            path: row.get(0)?,
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

fn change_to_dir(db_path: &PathBuf, limit: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Get recent directories from database
    let dirs: Vec<_> = recent_dirs(db_path, limit)?.into_iter().rev().collect();
    
    if dirs.is_empty() {
        eprintln!("No recent directories found in history");
        return Ok(());
    }
    
    // Create a list of directory paths for fzf, expanding to absolute paths
    // Use a HashSet to track seen paths and avoid duplicates
    let mut seen = HashSet::new();
    let mut dir_paths: Vec<String> = Vec::new();
    
    for d in &dirs {
        let path = PathBuf::from(&d.path);
        // Try to canonicalize the path to get absolute path
        // If it fails (directory doesn't exist), try expanding manually
        let abs_path_opt = match path.canonicalize() {
            Ok(abs_path) => Some(abs_path.to_string_lossy().to_string()),
            Err(_) => {
                // If canonicalize fails, try manual expansion
                if path.is_absolute() {
                    Some(d.path.clone())
                } else {
                    // Expand relative path from home directory
                    if let Ok(home) = env::var("HOME") {
                        let expanded = PathBuf::from(home).join(&path);
                        Some(expanded.to_string_lossy().to_string())
                    } else {
                        None
                    }
                }
            }
        };
        
        // Only add if we haven't seen this path before
        if let Some(abs_path) = abs_path_opt {
            if seen.insert(abs_path.clone()) {
                dir_paths.push(abs_path);
            }
        }
    }
    
    if dir_paths.is_empty() {
        eprintln!("No valid directories found in history");
        return Ok(());
    }
    
    // Launch fzf with the directory paths
    let mut fzf = Command::new("fzf")
        .arg("--height=40%")
        .arg("--reverse")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    
    // Send directory paths to fzf's stdin
    if let Some(mut stdin) = fzf.stdin.take() {
        for path in &dir_paths {
            writeln!(stdin, "{}", path)?;
        }
    }
    
    // Wait for fzf to finish and get the selected directory
    let output = fzf.wait_with_output()?;
    
    if output.status.success() {
        let selected_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if !selected_dir.is_empty() {
            let path = PathBuf::from(&selected_dir);
            
            // The path should already be absolute from our processing above,
            // but let's make sure it exists
            if path.exists() && path.is_dir() {
                // Print the selected directory path so it can be captured by a shell function
                println!("{}", selected_dir);
            } else {
                eprintln!("Selected directory no longer exists: {}", selected_dir);
                std::process::exit(1);
            }
        }
    } else {
        // User cancelled fzf (Ctrl+C or Escape)
        std::process::exit(1);
    }
    
    Ok(())
}

fn change_to_file(db_path: &PathBuf, limit: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Get recent files from database
    let files: Vec<_> = recent_files(db_path, limit)?.into_iter().rev().collect();
    
    if files.is_empty() {
        eprintln!("No recent files found in history");
        return Ok(());
    }
    
    // Create a list of file paths for fzf, expanding to absolute paths
    // Use a HashSet to track seen paths and avoid duplicates
    let mut seen = HashSet::new();
    let mut file_paths: Vec<String> = Vec::new();
    
    for f in &files {
        let path = PathBuf::from(&f.path);
        // Try to canonicalize the path to get absolute path
        // If it fails (file doesn't exist), try expanding manually
        let abs_path_opt = match path.canonicalize() {
            Ok(abs_path) => Some(abs_path.to_string_lossy().to_string()),
            Err(_) => {
                // If canonicalize fails, try manual expansion
                if path.is_absolute() {
                    Some(f.path.clone())
                } else {
                    // Expand relative path from home directory
                    if let Ok(home) = env::var("HOME") {
                        let expanded = PathBuf::from(home).join(&path);
                        Some(expanded.to_string_lossy().to_string())
                    } else {
                        None
                    }
                }
            }
        };
        
        // Only add if we haven't seen this path before
        if let Some(abs_path) = abs_path_opt {
            if seen.insert(abs_path.clone()) {
                file_paths.push(abs_path);
            }
        }
    }
    
    if file_paths.is_empty() {
        eprintln!("No valid files found in history");
        return Ok(());
    }
    
    // Launch fzf with the file paths
    let mut fzf = Command::new("fzf")
        .arg("--height=40%")
        .arg("--reverse")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    
    // Send file paths to fzf's stdin
    if let Some(mut stdin) = fzf.stdin.take() {
        for path in &file_paths {
            writeln!(stdin, "{}", path)?;
        }
    }
    
    // Wait for fzf to finish and get the selected file
    let output = fzf.wait_with_output()?;
    
    if output.status.success() {
        let selected_file = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if !selected_file.is_empty() {
            let path = PathBuf::from(&selected_file);
            
            // The path should already be absolute from our processing above,
            // but let's make sure it exists
            if path.exists() && path.is_file() {
                // Print the selected file path so it can be captured by a shell function
                println!("{}", selected_file);
            } else {
                eprintln!("Selected file no longer exists: {}", selected_file);
                std::process::exit(1);
            }
        }
    } else {
        // User cancelled fzf (Ctrl+C or Escape)
        std::process::exit(1);
    }
    
    Ok(())
}

fn print_json<T: Serialize>(data: &T, use_color: bool) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(data)?;
    
    if use_color {
        if std::io::stdout().is_terminal() {
            println!("{}", json_string.to_colored_json_auto()?);
        } else {
            println!("{}", json_string);
        }
    } else {
        println!("{}", json_string);
    }
    
    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  fzf-nav [--db-path <path>] [--no-color] recent-dirs [limit]     # Show recent directory visits (default: 50)");
    println!("  fzf-nav [--db-path <path>] [--no-color] recent-files [limit]    # Show recent file opens (default: 50)");
    println!("  fzf-nav [--db-path <path>] [--no-color] popular-dirs [limit]    # Show most visited directories (default: 50)");
    println!("  fzf-nav [--db-path <path>] [--no-color] file-stats              # Show file type statistics");
    println!("  fzf-nav [--db-path <path>] [--no-color] search <query>          # Search history");
    println!("  fzf-nav [--db-path <path>] change-to-dir [limit]                # Interactive directory selection with fzf (default: 100)");
    println!("  fzf-nav [--db-path <path>] change-to-file [limit]               # Interactive file selection with fzf (default: 100)");
    println!("  fzf-nav help                                                    # Show this help message");
    println!();
    println!("Options:");
    println!("  --db-path <path>    Path to the database file (default: ~/.fzf.db)");
    println!("  --no-color          Disable colored JSON output");
}

fn parse_args(args: &[String]) -> (Option<PathBuf>, bool, Vec<String>) {
    let mut db_path = None;
    let mut use_color = true;
    let mut remaining_args = Vec::new();
    let mut i = 1; // Skip program name
    
    while i < args.len() {
        match args[i].as_str() {
            "--db-path" => {
                if i + 1 < args.len() {
                    db_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2; // Skip both --db-path and its value
                } else {
                    eprintln!("Error: --db-path requires a value");
                    std::process::exit(1);
                }
            },
            "--no-color" => {
                use_color = false;
                i += 1;
            },
            _ => {
                remaining_args.push(args[i].clone());
                i += 1;
            }
        }
    }
    
    (db_path, use_color, remaining_args)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let (custom_db_path, use_color, remaining_args) = parse_args(&args);
    let db_path = custom_db_path.unwrap_or_else(get_default_db_path);
    
    if remaining_args.is_empty() {
        print_usage();
        return;
    }
    
    let result = match remaining_args[0].as_str() {
        "recent-dirs" => {
            let limit = remaining_args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(50);
            
            match recent_dirs(&db_path, limit) {
                Ok(dirs) => {
                    if let Err(e) = print_json(&dirs, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        
        "recent-files" => {
            let limit = remaining_args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(50);
            
            match recent_files(&db_path, limit) {
                Ok(files) => {
                    if let Err(e) = print_json(&files, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        
        "popular-dirs" => {
            let limit = remaining_args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(50);
            
            match popular_dirs(&db_path, limit) {
                Ok(dirs) => {
                    if let Err(e) = print_json(&dirs, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        
        "file-stats" => {
            match file_stats(&db_path) {
                Ok(stats) => {
                    if let Err(e) = print_json(&stats, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        
        "search" => {
            if remaining_args.len() < 2 {
                eprintln!("Error: search requires a query string");
                print_usage();
                return;
            }
            
            let query = &remaining_args[1];
            match search_history(&db_path, query) {
                Ok(results) => {
                    if let Err(e) = print_json(&results, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        
        "change-to-dir" => {
            let limit = remaining_args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            
            if let Err(e) = change_to_dir(&db_path, limit) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return; // Don't process result further since change_to_dir handles its own output
        },

        "change-to-file" => {
            let limit = remaining_args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            
            if let Err(e) = change_to_file(&db_path, limit) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return; // Don't process result further since change_to_file handles its own output
        },
        
        "help" | "--help" | "-h" => {
            print_usage();
            return;
        },
        
        _ => {
            eprintln!("Unknown command: {}", remaining_args[0]);
            print_usage();
            return;
        }
    };
    
    if let Err(e) = result {
        eprintln!("Database error: {}", e);
        eprintln!("Make sure the database exists at: {:?}", db_path);
    }
}
