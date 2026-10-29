use leptos::prelude::*;

#[component]
pub fn CustomButton(
    #[prop(into, default = Callback::new(|_: ()| {}))] on_click: Callback<()>,
    #[prop(into, default = String::from("Button"))] label: String,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <button class="btn" disabled=move || disabled.get() on:click=move |_| on_click.run(())>
            {label}
        </button>
    }
}