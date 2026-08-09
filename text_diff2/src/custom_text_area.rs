use crate::helper::*;
use crate::text_block;
use leptos::attr::any_attribute::AnyAttribute;
use leptos::prelude::*;

#[component]
pub fn CustomTextArea(
    on_diff_update: impl Fn(text_block::TextBlock, String) + 'static,
    #[prop(into)] text_block: text_block::TextBlock,
    /// Extra attributes (class, id, etc.) forwarded from wherever this component is used.
    #[prop(attrs)]
    attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    let node_ref = NodeRef::<leptos::html::Textarea>::new();

    view! {
        /*<textarea
            {..attrs}
            prop:value=move || current_text.get()
            on:input=move |ev| {
                let new_text = event_target_value(&ev);
                on_diff_update(text_block, new_text.clone()); //updates the parents db.
                update_diff(&text_block, new_text);
            }

            /*on:keydown=move |ev: web_sys::KeyboardEvent| {
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
            }*/

            node_ref=node_ref
        ></textarea>*/

        <textarea
            {..attrs}
            prop:value=move || text_block.text.0.get()
            on:input=move |ev| {
                let new_text = event_target_value(&ev);
                on_diff_update(text_block.clone(), new_text.clone());
                update_diff(&text_block, new_text);
            }
            node_ref=node_ref
        ></textarea>
    }
}
