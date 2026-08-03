use leptos::prelude::*;

#[derive(Clone)]
pub struct PendingConfirm {
    pub message: String,
    pub on_confirm: Callback<()>,
}

#[component]
pub fn ConfirmModal(pending_confirm: RwSignal<Option<PendingConfirm>>) -> impl IntoView {
    let confirm = move |_| {
        if let Some(pending) = pending_confirm.get() {
            pending.on_confirm.run(());
        }
        pending_confirm.set(None);
    };

    let cancel = move |_| {
        pending_confirm.set(None);
    };

    view! {
        <Show when=move || pending_confirm.get().is_some() fallback=|| ()>
            <div id="confirm-modal" class="modal-overlay">
                <div class="modal-box">
                    <p id="confirm-modal-text">
                        {move || pending_confirm.get().map(|p| p.message).unwrap_or_default()}
                    </p>
                    <div class="modal-actions">
                        <button id="confirm-modal-yes" class="danger" on:click=confirm>"Yes, delete"</button>
                        <button id="confirm-modal-no" on:click=cancel>"Cancel"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
