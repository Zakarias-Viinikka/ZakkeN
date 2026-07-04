use leptos::prelude::*;
#[component]
pub fn TextArea(index: ReadSignal<usize>, text: RwSignal<String>) -> impl IntoView {
    //
    view! {
        <li class="text-container" data-id={move || index.get()}>
            <div class="drag-handle">"⠿"</div>
            <div class="text-input-container">
                <textarea
                    id={move || index.get()}
                    class="textarea"
                    //prop:value=content
                    on:input=move |ev| {
                        text.set(event_target_value(&ev));
                    }
                    placeholder="Type something..."
                ></textarea>
            </div>
        </li>
    }
}
