//! Caller pov: what the reconcile page is currently doing. The
//! component matches on this to decide what to render.

use std::collections::HashMap;

use generate_love_letters::Block;

use crate::owner_of_interacting_with_server::ConnState;

#[derive(Debug, Clone)]
pub struct MismatchedPage {
    pub page_id: String,
    pub client_blob: Vec<u8>,
    pub client_blocks: Vec<Block>,
    pub server_blob: Vec<u8>,
    pub server_blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct ClientOnlyPage {
    pub page_id: String,
    pub client_blob: Vec<u8>,
    pub is_main_menu_page: bool,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Loading,
    NotLonewolf(ConnState),
    Ready {
        target_drift_version: String,
        target_reconcile_version: String,
        mismatched: Vec<MismatchedPage>,
        client_only: Vec<ClientOnlyPage>,
        mount_fingerprints: HashMap<String, String>,
        user_id: String,
        time: String,
    },
    Error(String),
}
