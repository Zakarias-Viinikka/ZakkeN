use protocol::serialization::Convert;
use server_errors::ServErr;
use server_protocol::{Block, ImportPagesPayload, ImportPagesReply, ImportedPage};
use std::io::Read;

/// Return one page-batch of the import zip named in the payload. Reads
/// the zip, not the db, so it needs no db access.
pub fn import_pages(msg_content: Vec<u8>) -> Vec<u8> {
    let result: Result<ImportPagesReply, ServErr> = (|| {
        let payload = ImportPagesPayload::un_payloadify(&msg_content)
            .map_err(ServErr::DbErrorContainer)?;
        read_a_page_of_the_import(
            &payload.zip_path,
            payload.after_page_id.as_deref(),
            payload.limit,
        )
    })();
    result.to_payload()
}

/// Read up to `limit` pages from the export zip, starting after
/// `after_page_id`. Pages are ordered by page_id, which is what makes
/// the cursor work.
fn read_a_page_of_the_import(
    zip_path: &str,
    after_page_id: Option<&str>,
    limit: usize,
) -> Result<ImportPagesReply, ServErr> {
    if limit == 0 {
        return Err(ServErr::BadCode("import limit must be at least 1".to_string()));
    }

    let file = std::fs::File::open(zip_path)
        .map_err(|e| ServErr::BadCode(format!("couldn't open {zip_path}: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| ServErr::BadCode(format!("{zip_path} isn't a valid zip: {e}")))?;

    let mut page_ids: Vec<String> = archive
        .file_names()
        .filter_map(|name| name.strip_suffix(".json"))
        .map(String::from)
        .collect();
    page_ids.sort();

    let remaining: Vec<String> = page_ids
        .into_iter()
        .filter(|id| after_page_id.map_or(true, |after| id.as_str() > after))
        .collect();
    let has_more = remaining.len() > limit;

    let mut pages = Vec::new();
    for page_id in remaining.into_iter().take(limit) {
        let entry_name = format!("{page_id}.json");
        let mut entry = archive
            .by_name(&entry_name)
            .map_err(|e| ServErr::BadCode(format!("couldn't open {entry_name} in zip: {e}")))?;
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|e| ServErr::BadCode(format!("couldn't read {entry_name}: {e}")))?;
        let blocks: Vec<Block> = serde_json::from_str(&text)
            .map_err(|e| ServErr::BadCode(format!("{entry_name} isn't valid page json: {e}")))?;
        pages.push(ImportedPage { page_id, blocks });
    }

    Ok(ImportPagesReply { pages, has_more })
}