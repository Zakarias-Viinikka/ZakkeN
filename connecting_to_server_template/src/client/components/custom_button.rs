use leptos::prelude::*;

#[component]
pub fn CustomButton(
    #[prop(into, default = Callback::new(|_: ()| {}))] on_click: Callback<()>,
    #[prop(into, default = String::from("Button"))] label: String,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    // Every button in the app respects the hard lock. Once locked, the
    // only way out is a reload, so no button should be clickable.
    let locked = crate::owner_of_interacting_with_server::locked::hard_locked_signal();
    view! {
        <button
            class="btn"
            disabled=move || disabled.get() || locked.get()
            on:click=move |_| on_click.run(())
        >
            {label}
        </button>
    }
}