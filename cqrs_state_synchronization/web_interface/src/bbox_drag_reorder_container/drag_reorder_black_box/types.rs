use leptos::prelude::{ArcRwSignal, Signal};

#[derive(Clone, Debug)]
pub struct PageInput {
    pub page_id: String,
}


#[derive(Clone, Debug)]
pub struct MoveEvent {
    pub moved_block_id: String,
    pub from_page_id: String,
    pub to_page_id: String,
    pub above: Option<String>,
    pub below: Option<String>,
}

#[derive(Clone)]
pub struct BlockRenderInput {
    pub block_id: String,
    pub content: ArcRwSignal<String>,
    pub ghost: Signal<bool>,
    pub is_title: bool,
}