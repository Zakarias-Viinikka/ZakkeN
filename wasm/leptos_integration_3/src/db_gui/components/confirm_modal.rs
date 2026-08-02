use leptos::prelude::*;

#[component]
pub fn ConfirmModal() -> impl IntoView {
    view! {
        <div id="confirm-modal" class="modal-overlay hidden">
            <div class="modal-box">
                <p id="confirm-modal-text"></p>
                <div class="modal-actions">
                    <button id="confirm-modal-yes" class="danger">"Yes, delete"</button>
                    <button id="confirm-modal-no">"Cancel"</button>
                </div>
            </div>
        </div>
    }
}
