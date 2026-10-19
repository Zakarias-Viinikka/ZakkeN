//! Signal flow for pages.

use leptos::prelude::*;

/// A block, as the whole app stores and passes it around.
#[derive(Clone, Debug)]
pub struct BlockRow {
    pub block_id: String,
    pub page_id: String,
    pub content: String,
    pub is_title: bool,
}

/// One page's entire live state.
///
/// Created once in MainContent, one per page. Copy-able, so effects
/// capture it by value without an Rc. Each page's effects read and
/// write only its own Page — page1's effects never touch page2's.
///
/// `page_state` drives the loading sequence. The state effect matches
/// on it, does one step, and lets the resulting state change fire the
/// effect again for the next step.
///
/// `preload_request` and `preload_response` are the preload exchange
/// for this page. Each page has its own pair so a slow response for
/// one page can't clobber the other's.
#[derive(Clone, Copy)]
pub struct Page {
    pub page_id: RwSignal<String>,
    pub boss_of_yrs: RwSignal<Option<std::sync::Arc<my_yrs_lib::BossOfYrs>>>,
    pub blocks: RwSignal<Vec<BlockRow>>,
    pub page_state: RwSignal<PageLoadState>,
    pub preload_request: RwSignal<Option<String>>,
    pub preload_response: RwSignal<Option<Vec<BlockRow>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageLoadState {
    Loading,
    /// The number is how many blocks the preloader returned. Zero when
    /// nothing was preloaded.
    PreloadLoaded(usize),
    Loaded,
    Online,
    Offline,
    LoadFailed,
}

impl Page {
    pub fn new(page_id: String) -> Self {
        Self {
            page_id: RwSignal::new(page_id),
            boss_of_yrs: RwSignal::new(None),
            blocks: RwSignal::new(Vec::new()),
            page_state: RwSignal::new(PageLoadState::Loading),
            preload_request: RwSignal::new(None),
            preload_response: RwSignal::new(None),
        }
    }
}

/// One row in the nav menu's page list.
#[derive(Clone)]
pub struct LocalPages {
    pub db_id: usize,
    pub title: String,
    pub block_id: String,
    pub page_id: String,
    pub is_disabled: bool,
}