use leptos::prelude::*;

#[derive(Copy, Clone)]
pub struct TextBlock {
    pub text: (ReadSignal<String>, WriteSignal<String>),
    pub id: RwSignal<f64>,
    pub latest_diff: RwSignal<String>,
}

impl TextBlock {
    pub fn new(text: String, id: RwSignal<f64>) -> Self {
        Self {
            text: signal(text),
            id,
            latest_diff: RwSignal::new("".to_string()),
        }
    }
}
