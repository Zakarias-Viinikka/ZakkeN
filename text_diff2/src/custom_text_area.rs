use crate::helper::*;
use crate::text_block;
use leptos::attr::any_attribute::AnyAttribute;
use leptos::prelude::*;

#[component]
pub fn CustomTextArea(
    /// Which box this is. Only used to label the diff so the parent knows where it came from.
    #[prop(into)]
    text_block: text_block::TextBlock,
    /// Extra attributes (class, id, etc.) forwarded from wherever this component is used.
    #[prop(attrs)]
    attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    let current_text = RwSignal::new(String::new());
    let node_ref = NodeRef::<leptos::html::Textarea>::new();

    view! {
        <textarea
            {..attrs}
            prop:value=move || current_text.get()
            on:input=move |ev| {
                save_latest_change(current_text, event_target_value(&ev));
                update_diff(&text_block, current_text.get());
            }

            on:keydown=move |ev: web_sys::KeyboardEvent| {
                /*
                 *
                 * field-sizing: content
                 * for making text areas behave properly in css easily
                 *
                 *
                 */
                // Tab never fires an input event, so we still need to flush
                // any pending diff. Re‑implement when the new diff logic is added.
                if ev.key() == "Tab" {
                    // TODO: flush any buffered changes for this box
                }
            }

            node_ref=node_ref
        ></textarea>
    }
}
