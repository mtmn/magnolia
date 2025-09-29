pub mod queries;
pub mod utils;

pub use queries::{file_stats, popular_dirs, recent_dirs, recent_files, search_history};
pub use utils::get_default_db_path;
