use crate::allowed_requests::err_bytes;
use db_core::db::operations_pages;
use db_wrapper::android_mascot::LiveForever;
use my_yrs_lib::doc_from_snapshot;
use protocol::serialization::Convert;
use serde::Serialize;
use server_errors::ServErr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const PATH: &str = "export_temp";
const ZIP_PATH: &str = "export_temp.zip";

#[derive(Serialize)]
struct Block {
    block_id: String,
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

    if let Err(e) = zip_folder() {
        return err_bytes(ServErr::BadCode(format!("couldn't zip export folder: {e}")));
    }

    let date_str = chrono::Local::now().format("%Y_%m_%d").to_string();
    let filename = format!("zakken_export_{date_str}.zip");

    let downloads_dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join("Downloads"),
        Err(_) => PathBuf::from("."),
    };

    let target_path = downloads_dir.join(&filename);
    if let Err(e) = std::fs::copy(ZIP_PATH, &target_path) {
        return err_bytes(ServErr::BadCode(format!(
            "couldn't copy zip to {}: {e}",
            target_path.display()
        )));
    }

    let _ = std::fs::remove_dir_all(PATH);
    let _ = std::fs::remove_file(ZIP_PATH);

    let ok: Result<(), ServErr> = Ok(());
    ok.to_payload()
}

fn zip_folder() -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = std::fs::File::create(ZIP_PATH)?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in std::fs::read_dir(PATH)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            zip.start_file(name, options)?;
            let mut file = std::fs::File::open(&path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn get_blocks_for_page(db: &LiveForever, page_id: &str) -> Result<Vec<Block>, ServErr> {
    let stored = operations_pages::get_page(db, page_id)
        .map_err(ServErr::DbErrorContainer)?
        .ok_or_else(|| ServErr::BadCode(format!("page {page_id} not found")))?;

    let boss = doc_from_snapshot(
        stored.blobbed_page,
        String::new(),
        page_id.to_string(),
        String::new(),
    )
    .map_err(|e| ServErr::BadCode(format!("failed to decode yrs doc for {page_id}: {e:?}")))?;

    let yrs_blocks = Arc::new(boss)
        .get_entire_page()
        .map_err(|e| ServErr::BadCode(format!("failed to read blocks for {page_id}: {e:?}")))?;

    let blocks = yrs_blocks
        .into_iter()
        .enumerate()
        .map(|(i, b)| Block {
            block_id: b.id_in_yrs,
            content: b.text,
            metadata: b.metadata,
            frac_index: i as i32,
        })
        .collect();

    Ok(blocks)
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
    let zip_path = Path::new(ZIP_PATH);
    if zip_path.exists() {
        std::fs::remove_file(zip_path)?;
    }
    std::fs::create_dir_all(path)?;
    Ok(())
}
