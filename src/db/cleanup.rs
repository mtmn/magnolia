use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};

pub fn cleanup_database(db_path: &PathBuf) -> Result<()> {
    let mut conn = Connection::open(db_path)?;
    let tx = conn.transaction()?;
    
    // Cleanup directories
    let dirs_to_remove = {
        let mut stmt = tx.prepare("SELECT id, path FROM directory_history")?;
        let dir_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut to_remove = Vec::new();
        for entry in dir_iter {
            let (id, path) = entry?;
            if !Path::new(&path).exists() {
                to_remove.push(id);
            }
        }
        to_remove
    };

    if !dirs_to_remove.is_empty() {
        let mut delete_dir = tx.prepare("DELETE FROM directory_history WHERE id = ?")?;
        for id in dirs_to_remove {
            delete_dir.execute([id])?;
        }
    }
    
    // Cleanup files
    let files_to_remove = {
        let mut stmt = tx.prepare("SELECT id, path FROM file_history")?;
        let file_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut to_remove = Vec::new();
        for entry in file_iter {
            let (id, path) = entry?;
            if !Path::new(&path).exists() {
                to_remove.push(id);
            }
        }
        to_remove
    };

    if !files_to_remove.is_empty() {
        let mut delete_file = tx.prepare("DELETE FROM file_history WHERE id = ?")?;
        for id in files_to_remove {
            delete_file.execute([id])?;
        }
    }

    tx.commit()?;
    
    Ok(())
}
