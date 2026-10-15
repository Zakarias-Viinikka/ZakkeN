use crate::server_state::ServerState;
use db_core::db::operations_pages;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{GetAllYrsPagesPayload, PageRow};

pub fn handle(_msg_content: Vec<u8>, state: &mut ServerState) -> (Vec<u8>, Option<String>) {
    let stored = match operations_pages::get_all_pages(&state.liver) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<GetAllYrsPagesPayload, DbError> = Err(e);
            return (wrapped.to_payload(), None);
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

    let wrapped: Result<GetAllYrsPagesPayload, DbError> =
        Ok(GetAllYrsPagesPayload { pages });
    (wrapped.to_payload(), None)
}