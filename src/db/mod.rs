pub mod queries;
pub mod utils;
pub mod cleanup;

pub use queries::{file_stats, frequent_dirs, recent_dirs, recent_files, search_history};
pub use cleanup::cleanup_database;
pub use utils::get_default_db_path;
