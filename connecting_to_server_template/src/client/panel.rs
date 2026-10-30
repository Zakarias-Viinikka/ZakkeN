use crate::client::status_line::{reason_label, status_line};
use crate::client_server_coms::ComsHandleForClient;
use crate::CustomButton;
use leptos::prelude::*;

#[component]
pub fn ClientPanel() -> impl IntoView {
    let handle = expect_context::<ComsHandleForClient>();
    let conn = crate::server_link::conn_state::conn_state_signal();
    let mode = crate::sync_mode::sync_mode_signal();
    let reason = crate::sync_mode::mode_reason_signal();
    let pending = crate::server_link::pending_queue::pending_count_signal();
    let applied = crate::server_link::receiving::applied_count_signal();
    let ignored = crate::server_link::receiving::ignored_while_stopped_signal();
    let curr_gen = crate::server_link::dataset_generation::current_generation_signal();
    let stored_gen = crate::server_link::dataset_generation::stored_generation_signal();
    let status = status_line();
    let handle_a = handle.clone();
    let handle_b = handle;

    let gen_mismatch = crate::server_link::dataset_generation::generations_differ_signal();
    let gm1 = gen_mismatch.clone();
    let gm2 = gen_mismatch.clone();
    let curr_b = curr_gen.clone();
    let stored_b = stored_gen.clone();

    view! {
        <p style="font-weight:600; margin-bottom:0.75rem;">{move || status.get()}</p>
        <p>"mode: " {move || format!("{:?}", mode.get())}
            {move || reason.get().map(|r| format!(" ({})", reason_label(Some(r))))}
        </p>
        <p>"conn: " {move || format!("{:?}", conn.get())}</p>
        <p>"pending: " {move || pending.get()}</p>
        <p>"applied: " {move || applied.get()}</p>
        <p>"ignored while stopped: " {move || ignored.get()}</p>
        <p style=move || if gm1.get() { "color:#e8b060;" } else { "" }>
            "login gen: " {move || format!("{:?}", curr_b.get())}
        </p>
        <p style=move || if gm2.get() { "color:#e8b060;" } else { "" }>
            "stored gen: " {move || format!("{:?}", stored_b.get())}
        </p>
        <CustomButton
            label="Pretend edit page_a"
            on_click=move |_| {
                crate::server_link::pending_queue::queue_pretend_edit(&handle_a, "page_a", "edited a");
            }
        />
        <CustomButton
            label="Pretend edit page_b"
            on_click=move |_| {
                crate::server_link::pending_queue::queue_pretend_edit(&handle_b, "page_b", "edited b");
            }
        />
    }
}
