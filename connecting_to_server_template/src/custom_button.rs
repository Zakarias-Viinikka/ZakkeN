use leptos::prelude::*;

#[component]
pub fn CustomButton(
    #[prop(into, default = Callback::new(|_: ()| {}))] on_click: Callback<()>,
    #[prop(into, default = String::from("Button"))] label: String,
) -> impl IntoView {
    view! {
        <button class="btn" on:click=move |_| on_click.run(())>
            {label}
        </button>
    }
}