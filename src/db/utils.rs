use std::env;
use std::path::PathBuf;

/// See: https://codeberg.org/mtmn/dotfiles/src/branch/master/dot_config/nvim/fnl/functions.fnl#L45
pub fn normalize_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("oil://") {
        stripped.to_string()
    } else {
        path.to_string()
    }
}

pub fn get_default_db_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".magnolia.db")
}
