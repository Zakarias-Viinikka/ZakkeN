use leptos::prelude::*;

use crate::checkbox_logic::Checkbox;

//let (checked, set_checked, _) = use_local_storage::<bool, JsonSerdeCodec>("seed-blocks");
#[component]
pub fn HappyLittleCheckbox(checkbox: RwSignal<Checkbox>) -> impl IntoView {
    view! {
        <input type="checkbox"
            prop:checked=checkbox.get().is_active
            on:change=move |ev| {
                let new_check_state = event_target_checked(&ev);
                checkbox.update(|checkbox| {
                    checkbox.is_active_set.set(new_check_state);
                });
            }
        />
    }
}
