use std::env;
use std::path::PathBuf;

pub fn get_default_db_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".magnolia.db")
}
