use leptos::prelude::*;

//let (checked, set_checked, _) = use_local_storage::<bool, JsonSerdeCodec>("seed-blocks");
#[component]
pub fn HappyLittleCheckbox(
    box_is_checked: ReadSignal<bool>,
    box_is_checked_set: WriteSignal<bool>,
    speak_your_truth: impl Fn() + 'static,
) -> impl IntoView {
    Effect::new(move |_| {
        if box_is_checked.get_untracked() {
            speak_your_truth();
        }
    });
    view! {
        <input type="checkbox"
            prop:checked=box_is_checked
            on:change=move |ev| box_is_checked_set.set(event_target_checked(&ev))
        />
    }
}
