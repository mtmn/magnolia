use crate::db::queries::{recent_dirs, recent_files};
use std::collections::HashSet;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn change_to_dir(db_path: &PathBuf, limit: i32) -> Result<(), Box<dyn std::error::Error>> {
    let dirs: Vec<_> = recent_dirs(db_path, limit)?.into_iter().rev().collect();

    if dirs.is_empty() {
        eprintln!("No recent directories found in history");
        return Ok(());
    }

    let mut seen = HashSet::new();
    let mut dir_paths: Vec<String> = Vec::new();

    for d in &dirs {
        let path = PathBuf::from(&d.path);
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

    let mut fzf = Command::new("fzf")
        .arg("--height=40%")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

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

            if path.exists() && path.is_dir() {
                println!("{}", selected_dir);
            } else {
                eprintln!("Selected directory no longer exists: {}", selected_dir);
                std::process::exit(1);
            }
        }
    } else {
        std::process::exit(1);
    }

    Ok(())
}

pub fn change_to_file(db_path: &PathBuf, limit: i32) -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = recent_files(db_path, limit)?.into_iter().rev().collect();

    if files.is_empty() {
        eprintln!("No recent files found in history");
        return Ok(());
    }

    let mut seen = HashSet::new();
    let mut file_paths: Vec<String> = Vec::new();

    for f in &files {
        let path = PathBuf::from(&f.path);
        let abs_path_opt = match path.canonicalize() {
            Ok(abs_path) => Some(abs_path.to_string_lossy().to_string()),
            Err(_) => {
                if path.is_absolute() {
                    Some(f.path.clone())
                } else {
                    if let Ok(home) = env::var("HOME") {
                        let expanded = PathBuf::from(home).join(&path);
                        Some(expanded.to_string_lossy().to_string())
                    } else {
                        None
                    }
                }
            }
        };

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

    let mut fzf = Command::new("fzf")
        .arg("--height=40%")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = fzf.stdin.take() {
        for path in &file_paths {
            writeln!(stdin, "{}", path)?;
        }
    }

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
