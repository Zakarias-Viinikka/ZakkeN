//! Caller pov: fake mismatched pages plus the per-pair state container.
//! `PagePair` holds both sides, the picked side (if any), and the last
//! edited kept-side blocks (set by Done or X, cleared by Reset).

use generate_love_letters::Block;

use crate::pick_screen::PickedSide;

#[derive(Clone)]
pub struct PagePair {
    pub page_id: String,
    pub client_page: Vec<Block>,
    pub server_page: Vec<Block>,
    pub picked: Option<PickedSide>,
    pub edited_kept_blocks: Option<Vec<Block>>,
}

fn b(id: &str, text: &str, frac: &str) -> Block {
    Block {
        block_id: id.to_string(),
        text: text.to_string(),
        metadata: String::new(),
        frac_position: frac.to_string(),
    }
}

pub fn placeholder_items() -> Vec<PagePair> {
    vec![
        PagePair {
            page_id: "page-notes".to_string(),
            client_page: vec![
                b("bn1", "Release notes", "80"),
                b("bn2", "Version 0.3.0 released today.", "81"),
                b("bn3", "Highlights", "82"),
                b("bn4", "Faster startup, cleaner menus, better search.", "83"),
                b("bn5", "Thanks for reading.", "84"),
            ],
            server_page: vec![
                b("bn1", "Release notes", "80"),
                b("bn2", "Version 0.4.0 released today.", "81"),
                b("bn3", "Highlights", "82"),
                b("bn4", "Faster startup, cleaner menus, better search.", "83"),
                b("bn5", "Thanks for reading.", "84"),
            ],
            picked: None,
            edited_kept_blocks: None,
        },
        PagePair {
            page_id: "page-todo".to_string(),
            client_page: vec![
                b("bt1", "Todo list", "80"),
                b("bt2", "Buy milk", "81"),
                b("bt3", "Walk the dog", "82"),
                b("bt4", "Call mom", "83"),
            ],
            server_page: vec![
                b("bt1", "Todo list", "80"),
                b("bt2", "Buy milk", "81"),
                b("bt3", "Walk the dog", "82"),
                b("bt4", "Call mom", "83"),
                b("bt5", "Pick up dry cleaning", "84"),
            ],
            picked: None,
            edited_kept_blocks: None,
        },
        PagePair {
            page_id: "page-shopping".to_string(),
            client_page: vec![
                b("bs1", "Shopping list", "80"),
                b("bs4", "Apples", "805"),
                b("bs2", "Bread", "81"),
                b("bs3", "Cheese", "82"),
            ],
            server_page: vec![
                b("bs1", "Shopping list", "80"),
                b("bs2", "Bread", "81"),
                b("bs3", "Cheese", "82"),
                b("bs4", "Apples", "83"),
            ],
            picked: None,
            edited_kept_blocks: None,
        },
    ]
}