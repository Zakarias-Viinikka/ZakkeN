use crate::protocol::AdoptPage;

// PLACEHOLDER: the real `read_local_pages_placeholder` reads every local
// page row (blobbed_page) from the db. Here it returns canned pages.
pub fn read_local_pages_placeholder() -> Vec<AdoptPage> {
    use crate::protocol::{CANNED_PAGE_ID_A, CANNED_PAGE_ID_B};
    vec![
        AdoptPage {
            page_id: CANNED_PAGE_ID_A.to_string(),
            is_main_menu_page: true,
            snapshot: b"local_a".to_vec(),
        },
        AdoptPage {
            page_id: CANNED_PAGE_ID_B.to_string(),
            is_main_menu_page: false,
            snapshot: b"local_b".to_vec(),
        },
    ]
}

// PLACEHOLDER: the real apply path wipes every local page row, then builds
// each server page locally via build_merge_new_page inside one transaction.
// Here it only logs the count.
pub fn apply_server_pages_locally(pages: &[crate::protocol::PageRow]) {
    leptos::logging::log!(
        "apply_server_pages_locally: applying {} pages",
        pages.len()
    );
}
