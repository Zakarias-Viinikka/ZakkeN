use leptos::prelude::*;

#[derive(Copy, Clone)]
pub struct TextBlock {
    pub text: (ReadSignal<String>, WriteSignal<String>),
    pub id: RwSignal<usize>,
    pub latest_diff: RwSignal<String>,
}

impl TextBlock {
    pub fn new(text: String, id: RwSignal<usize>) -> Self {
        Self {
            text: signal(text),
            id,
            latest_diff: RwSignal::new("".to_string()),
        }
    }
}
