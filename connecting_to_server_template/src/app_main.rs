use crate::client::Client;
use crate::client_server_coms;
use crate::cool_popup::CoolPopup;
use crate::controls::Controls;
use crate::server::Server;
use leptos::prelude::*;

#[component]
pub fn AppMain() -> impl IntoView {
    let (client_started, set_client_started) = signal(false);
    let (server_started, set_server_started) = signal(false);
    let (client_handle, server_handle, client_rx, server_rx) = client_server_coms::new();
    let client_handle_for_controls = client_handle.clone();
    view! {
        <div class="sides">
            <Client handle=client_handle rx=client_rx>
                <Show
                    when=move || client_started.get()
                    fallback=move || view! {
                        <ClientFallback on_start=move |_| set_client_started.set(true)/>
                    }
                >
                    <div class="side">
                        <h2 class="side-title">"Client"</h2>
                        <div class="side-content">
                            <crate::CustomButton/>
                        </div>
                    </div>
                </Show>
            </Client>
            <Server handle=server_handle rx=server_rx>
                <Show
                    when=move || server_started.get()
                    fallback=move || view! {
                        <ServerFallback on_start=move |_| set_server_started.set(true)/>
                    }
                >
                    <div class="side">
                        <h2 class="side-title">"Server"</h2>
                        <div class="side-content"></div>
                    </div>
                </Show>
            </Server>
            <Controls coms=client_handle_for_controls/>
        </div>
        <CoolPopup/>
    }
}

#[component]
fn ClientFallback(#[prop(into)] on_start: Callback<()>) -> impl IntoView {
    view! {
        <div class="side-empty">
            <crate::CustomButton label="Start client" on_click=on_start/>
        </div>
    }
}

#[component]
fn ServerFallback(#[prop(into)] on_start: Callback<()>) -> impl IntoView {
    view! {
        <div class="side-empty">
            <crate::CustomButton label="Start server" on_click=on_start/>
        </div>
    }
}