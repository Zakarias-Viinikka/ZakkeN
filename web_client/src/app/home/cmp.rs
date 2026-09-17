use leptos::prelude::*;
use popup::popup;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <h1>"web_client"</h1>

        <button on:click=move |_| {
            popup::create_popup("Connect clicked".to_string());
        }>"Connect"</button>

        <br/>

        <button on:click=move |_| {
            popup::create_popup("Ping clicked".to_string());
        }>"Ping"</button>

        <br/>

        <button on:click=move |_| {
            popup::create_popup("Get All Pages clicked".to_string());
        }>"Get All Pages"</button>
    }
}
