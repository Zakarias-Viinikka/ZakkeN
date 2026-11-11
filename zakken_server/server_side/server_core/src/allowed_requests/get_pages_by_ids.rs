use db_core::db::operations_pages;
use db_wrapper::android_mascot::LiveForever;
use protocol::serialization::Convert;
use server_errors::ServErr;
use server_protocol::{GetAllYrsPagesPayload, GetPagesByIdsPayload, PageRow};

/// Return the full data of each requested page. Unknown ids are skipped
/// silently. Exempt from the drift check.
pub fn get_pages_by_ids(msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let result: Result<GetAllYrsPagesPayload, ServErr> = (|| {
        let payload: GetPagesByIdsPayload = GetPagesByIdsPayload::un_payloadify(&msg_content)
            .map_err(ServErr::DbErrorContainer)?;
        let mut pages = Vec::with_capacity(payload.page_ids.len());
        for page_id in payload.page_ids {
            match operations_pages::get_page(state, &page_id)? {
                Some(p) => pages.push(PageRow {
                    page_id: p.page_id,
                    blobbed_page: p.blobbed_page,
                    page_status: p.page_status,
                    version: p.version,
                    is_main_menu_page: p.is_main_menu_page,
                }),
                None => {}
            }
        }
        Ok(GetAllYrsPagesPayload { pages })
    })();
    result.to_payload()
}
