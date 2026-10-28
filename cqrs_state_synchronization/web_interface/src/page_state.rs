use std::sync::Arc;

use leptos::prelude::*;
use my_yrs_lib::BossOfYrs;

use crate::complicated_signal_flow::BlockRow;

#[derive(Clone, Copy)]
pub struct Page {
    pub page_id: RwSignal<String>,
    pub boss_of_yrs: RwSignal<Option<Arc<BossOfYrs>>>,
    pub blocks: RwSignal<Vec<BlockRow>>,
    pub page_state: RwSignal<PageLoadState>,
    pub preload_request: RwSignal<Option<String>>,
    pub preload_response: RwSignal<Option<Vec<BlockRow>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageLoadState {
    Loading,
    PreloadLoaded(usize),
    Loaded,
    Online,
    Offline,
    LoadFailed,
}

#[derive(Clone, Copy)]
pub struct OpenPages {
    pub page1: Page,
    pub page2: RwSignal<Option<Page>>,
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

impl OpenPages {
    pub fn boss_for_page(&self, page_id: &str) -> Option<Arc<BossOfYrs>> {
        if self.page1.page_id.get_untracked() == page_id {
            return self.page1.boss_of_yrs.get_untracked();
        }
        if let Some(p2) = self.page2.get_untracked() {
            if p2.page_id.get_untracked() == page_id {
                return p2.boss_of_yrs.get_untracked();
            }
        }
        None
    }

    pub fn page_for_block(&self, block_id: &str) -> Option<Page> {
        if self
            .page1
            .blocks
            .get_untracked()
            .iter()
            .any(|b| b.block_id == block_id)
        {
            return Some(self.page1);
        }
        if let Some(p2) = self.page2.get_untracked() {
            if p2
                .blocks
                .get_untracked()
                .iter()
                .any(|b| b.block_id == block_id)
            {
                return Some(p2);
            }
        }
        None
    }

    pub fn page_and_boss_for_block(&self, block_id: &str) -> Option<(Arc<BossOfYrs>, Page)> {
        let page = self.page_for_block(block_id)?;
        let boss = page.boss_of_yrs.get_untracked()?;
        Some((boss, page))
    }
}