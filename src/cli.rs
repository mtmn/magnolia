use colored_json::prelude::*;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::PathBuf;

pub fn print_usage() {
    println!("Usage:");
    println!("  magnolia [--db-path <path>] [--no-color] recent-dirs [limit]     # Show recent directory visits (default: 500)");
    println!("  magnolia [--db-path <path>] [--no-color] recent-files [limit]    # Show recent file opens (default: 500)");
    println!("  magnolia [--db-path <path>] [--no-color] popular-dirs [limit]    # Show most visited directories (default: 500)");
    println!("  magnolia [--db-path <path>] [--no-color] file-stats              # Show file type statistics");
    println!("  magnolia [--db-path <path>] [--no-color] search <query>          # Search history");
    println!("  magnolia [--db-path <path>] change-to-dir [limit]                # Interactive directory selection with fzf (default: 1000)");
    println!("  magnolia [--db-path <path>] change-to-file [limit]               # Interactive file selection with fzf (default: 1000)");
    println!("  magnolia help                                                    # Show this help message");
    println!();
    println!("Options:");
    println!("  --db-path <path>    Path to the database file (default: ~/.magnolia.db)");
    println!("  --no-color          Disable colored JSON output");
}

pub fn parse_args(args: &[String]) -> (Option<PathBuf>, bool, Vec<String>) {
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
            }
            "--no-color" => {
                use_color = false;
                i += 1;
            }
            _ => {
                remaining_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    (db_path, use_color, remaining_args)
}

pub fn print_json<T: Serialize>(
    data: &T,
    use_color: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
