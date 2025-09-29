mod cli;
mod db;
mod interactive;
mod models;

use cli::{parse_args, print_json, print_usage};
use db::{
    file_stats, get_default_db_path, popular_dirs, recent_dirs, recent_files, search_history,
};
use interactive::{change_to_dir, change_to_file};
use std::env;

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
            let limit = remaining_args
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(500);

            match recent_dirs(&db_path, limit) {
                Ok(dirs) => {
                    if let Err(e) = print_json(&dirs, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        "recent-files" => {
            let limit = remaining_args
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(500);

            match recent_files(&db_path, limit) {
                Ok(files) => {
                    if let Err(e) = print_json(&files, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        "popular-dirs" => {
            let limit = remaining_args
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(500);

            match popular_dirs(&db_path, limit) {
                Ok(dirs) => {
                    if let Err(e) = print_json(&dirs, use_color) {
                        eprintln!("JSON output error: {}", e);
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        "file-stats" => match file_stats(&db_path) {
            Ok(stats) => {
                if let Err(e) = print_json(&stats, use_color) {
                    eprintln!("JSON output error: {}", e);
                }
                Ok(())
            }
            Err(e) => Err(e),
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
                }
                Err(e) => Err(e),
            }
        }

        "change-to-dir" => {
            let limit = remaining_args
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);

            if let Err(e) = change_to_dir(&db_path, limit) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return;
        }

        "change-to-file" => {
            let limit = remaining_args
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);

            if let Err(e) = change_to_file(&db_path, limit) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return;
        }

        "help" | "--help" | "-h" => {
            print_usage();
            return;
        }

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
