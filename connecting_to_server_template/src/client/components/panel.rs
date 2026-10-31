use crate::client::status_line::status_line;
use crate::client_server_coms::ComsHandleForClient;
use crate::CustomButton;
use leptos::prelude::*;

#[component]
pub fn ClientPanel() -> impl IntoView {
    let handle = expect_context::<ComsHandleForClient>();
    let pending = crate::owner_of_interacting_with_server::pending_queue::pending_count_signal();
    let applied = crate::owner_of_interacting_with_server::receiving::applied_count_signal();
    let ignored = crate::owner_of_interacting_with_server::receiving::ignored_while_stopped_signal();
    let curr_gen = crate::owner_of_interacting_with_server::dataset_generation::current_generation_signal();
    let stored_gen = crate::owner_of_interacting_with_server::dataset_generation::stored_generation_signal();
    let status = status_line();
    let handle_add = handle.clone();
    let handle_edit = handle;
    let next_page_id = RwSignal::new(1i64);

    let gen_mismatch =
        crate::owner_of_interacting_with_server::dataset_generation::generations_differ_signal();
    let gen_style = move || {
        if gen_mismatch.get() {
            "font-size:0.85rem; color:#e8b060; margin:0;"
        } else {
            "font-size:0.85rem; color:#5a6270; margin:0;"
        }
    };
    let gen_text = move || {
        let c = curr_gen.get();
        let s = stored_gen.get();
        let fmt = |g: Option<i64>| match g {
            Some(n) => n.to_string(),
            None => "—".to_string(),
        };
        format!("gen {} → stored {}", fmt(c), fmt(s))
    };

    view! {
        <p style="font-weight:600; margin:0;">{move || status.get()}</p>
        <p style="font-size:0.9rem; color:#5a6270; margin:0;">
            {move || format!(
                "pending {} · applied {} · ignored {}",
                pending.get(),
                applied.get(),
                ignored.get(),
            )}
        </p>
        <p style=gen_style>{gen_text}</p>
        <CustomButton
            label="Pretend add page"
            on_click=move |_| {
                let id = next_page_id.get_untracked();
                next_page_id.set(id + 1);
                crate::owner_of_interacting_with_server::pending_queue::queue_create_page(
                    &handle_add,
                    format!("pretend_page_{id}"),
                );
            }
        />
        <CustomButton
            label="Pretend make edit"
            on_click=move |_| {
                crate::owner_of_interacting_with_server::pending_queue::queue_pretend_edit(
                    &handle_edit,
                    "page_a",
                    "edited",
                );
            }
        />
    }
}