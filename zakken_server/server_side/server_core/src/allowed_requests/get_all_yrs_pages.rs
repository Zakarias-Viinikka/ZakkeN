use db_core::db::operations_pages;
use db_wrapper::android_mascot::LiveForever;
use server_errors::ServErr;
use protocol::serialization::Convert;
use server_protocol::{GetAllYrsPagesPayload, PageRow};

pub fn send_all_pages(_msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let stored = match operations_pages::get_all_pages(state) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<GetAllYrsPagesPayload, ServErr> =
                Err(ServErr::DbErrorContainer(e));
            return wrapped.to_payload();
        }
    };

    let pages: Vec<PageRow> = stored
        .into_iter()
        .map(|p| PageRow {
            page_id: p.page_id,
            blobbed_page: p.blobbed_page,
            page_status: p.page_status,
            version: p.version,
            is_main_menu_page: p.is_main_menu_page,
        })
        .collect();

    let wrapped: Result<GetAllYrsPagesPayload, ServErr> =
        Ok(GetAllYrsPagesPayload { pages });
    wrapped.to_payload()
}