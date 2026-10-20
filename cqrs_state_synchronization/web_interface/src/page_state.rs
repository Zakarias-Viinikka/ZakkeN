//! One page's live state.
//!
//! Passed to `MainContent` (which runs its load effects and builds the
//! drag view from it), to `page_loading_logic` (which reads the page id
//! and fills in the boss, blocks, and state), and to nothing else.
//! The drag component gets the page id and the block list as plain
//! values, not the Page itself.

use leptos::prelude::*;

use crate::complicated_signal_flow::BlockRow;

/// One page's entire live state.
///
/// Created once in `main.rs` and handed to `MainContent` via context.
/// Not created by `MainContent` - the point is that a route change
/// reuses the same `Page` and just resets its fields, so the yrs doc
/// and the block list survive long enough for the flush-on-load to
/// matter.
///
/// `page_state` drives the loading sequence. The state effect in
/// `MainContent` matches on it, does one step, and lets the resulting
/// state change fire the effect again for the next step.
///
/// `preload_request` and `preload_response` are the preload exchange
/// for this page. Each page has its own pair so a slow response for
/// one can't clobber the other's once two-page view exists.
#[derive(Clone, Copy)]
pub struct Page {
    pub page_id: RwSignal<String>,
    pub boss_of_yrs: RwSignal<Option<std::sync::Arc<my_yrs_lib::BossOfYrs>>>,
    pub blocks: RwSignal<Vec<BlockRow>>,
    pub page_state: RwSignal<PageLoadState>,
    pub preload_request: RwSignal<Option<String>>,
    pub preload_response: RwSignal<Option<Vec<BlockRow>>>,
}

/// Where a page is in the preload -> offline -> server sequence.
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
