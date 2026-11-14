use crate::allowed_requests::err_bytes;
use db_core::db::operations_pages;
use db_wrapper::android_mascot::LiveForever;
use serde::Serialize;
use server_errors::ServErr;
use std::path::Path;

const PATH: &str = "export_temp";

#[derive(Serialize)]
struct Block {
    content: String,
    metadata: String,
    frac_index: i32,
}

pub fn export_all_pages(db: &LiveForever) -> Vec<u8> {
    if let Err(e) = clear_existing_import_folder() {
        return err_bytes(ServErr::BadCode(format!(
            "couldn't clear export folder: {e}"
        )));
    }

    let pg_ids = match get_all_page_ids(db) {
        Ok(ids) => ids,
        Err(e) => return err_bytes(e),
    };

    for id in &pg_ids {
        let blocks = match get_blocks_for_page(db, id) {
            Ok(b) => b,
            Err(e) => return err_bytes(e),
        };
        if let Err(e) = create_file(id, &blocks) {
            return err_bytes(ServErr::BadCode(format!(
                "couldn't create file for page {id}: {e}"
            )));
        }
    }

    // zip PATH and read the bytes back
    todo!()
}

fn get_blocks_for_page(db: &LiveForever, page_id: &str) -> Result<Vec<Block>, ServErr> {
    // unpack blobbed_page for page_id into blocks
    todo!()
}

fn create_file(page_id: &str, blocks: &[Block]) -> std::io::Result<()> {
    let path = Path::new(PATH).join(format!("{page_id}.json"));
    let json = serde_json::to_string_pretty(blocks)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)?;
    Ok(())
}

fn get_all_page_ids(db: &LiveForever) -> Result<Vec<String>, ServErr> {
    let stored = operations_pages::get_all_pages(db).map_err(ServErr::DbErrorContainer)?;
    Ok(stored.into_iter().map(|p| p.page_id).collect())
}

fn clear_existing_import_folder() -> std::io::Result<()> {
    let path = Path::new(PATH);
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir_all(path)?;
    Ok(())
}