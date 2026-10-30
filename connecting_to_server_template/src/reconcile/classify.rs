use crate::protocol::{
    AdoptPage, PageRow, CANNED_PAGE_ID_A, CANNED_PAGE_ID_B, CANNED_PAGE_ID_C,
};
use crate::reconcile::plan::{PageDiffSummary, ReconcilePlan};

// PLACEHOLDER: the real `classify_pages` diffes each page's content and
// orders rows by (frac, id) on both sides. Here it branches on whether the
// server returned anything, so both modal shapes can be exercised.
pub fn classify_pages_placeholder(_local: &[AdoptPage], server: &[PageRow]) -> ReconcilePlan {
    if server.is_empty() {
        return ReconcilePlan {
            take_from_server: Vec::new(),
            push_to_server: vec![CANNED_PAGE_ID_A.to_string(), CANNED_PAGE_ID_B.to_string()],
            identical: Vec::new(),
            disable_locally: Vec::new(),
            disable_on_server: Vec::new(),
            conflicts: Vec::new(),
        };
    }
    ReconcilePlan {
        take_from_server: Vec::new(),
        push_to_server: Vec::new(),
        identical: vec![CANNED_PAGE_ID_C.to_string()],
        disable_locally: Vec::new(),
        disable_on_server: Vec::new(),
        conflicts: vec![
            PageDiffSummary {
                page_id: CANNED_PAGE_ID_A.to_string(),
                client_title: "Mine A".into(),
                server_title: "Server A".into(),
                only_client: 1,
                only_server: 0,
                changed: 2,
                moved: 1,
            },
            PageDiffSummary {
                page_id: CANNED_PAGE_ID_B.to_string(),
                client_title: "Mine B".into(),
                server_title: "Server B".into(),
                only_client: 0,
                only_server: 2,
                changed: 1,
                moved: 0,
            },
        ],
    }
}
