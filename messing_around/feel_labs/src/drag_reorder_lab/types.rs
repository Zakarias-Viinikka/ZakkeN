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

#[derive(Clone, Debug)]
pub struct BlockInputEvent {
    pub block_id: String,
    pub old_text: String,
    pub new_text: String,
    pub was_enter: bool,
    pub cursor_pos: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockRow {
    pub block_id: String,
    pub page_id: String,
    pub content: String,
    pub is_title: bool,
}
