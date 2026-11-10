use leptos::prelude::*;

use crate::owner_of_interacting_with_server::{ConnState, CONN_STATE};

#[component]
pub fn AppBorder(children: Children) -> impl IntoView {
    let color = move || {
        match CONN_STATE.get() {
            ConnState::Offline => "#2255cc",
            ConnState::Connecting => "#aa8800",
            ConnState::LoggingIn => "#aa8800",
            ConnState::FailedToConnect => "#2255cc",
            ConnState::Connected => "#227722",
            ConnState::TwoTabs => "#444",
            ConnState::Lonewolf => "#2255cc",
            ConnState::LoneWolfAndOffline => "#2255cc",
        }
    };
    view! {
        <div style=move || {
            format!(
                "min-height: 100vh; box-sizing: border-box; border: 4px solid {};",
                color()
            )
        }>
            <div style="padding: 1rem; padding-left: 72px;">
                {children()}
            </div>
        </div>
    }
}
