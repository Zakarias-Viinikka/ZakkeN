use leptos::prelude::*;

use crate::{
    db::ui_actions::create_new_page, leptos_components::small_components::popup::create_popup,
    shared_structs::LocalPages,
};

pub async fn insert_three_pages(
    ctr: RwSignal<usize>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) {
    for _ in 0..3 {
        if let Err(e) = create_new_page(ctr, local_pages_set).await {
            create_popup(format!("Error: {:?}", e));
            return;
        }
    }
}
