use leptos::prelude::*;

use crate::owner_of_interacting_with_server::{ConnState, CONN_STATE, PENDING_COUNT};

#[component]
pub fn ConnectionIndicator() -> impl IntoView {
    let bg = move || {
        let state = CONN_STATE.get();
        let pending = PENDING_COUNT.get();
        match state {
            ConnState::Offline => "#444",
            ConnState::Connecting => "#aa8800",
            ConnState::FailedToConnect => "#2255cc",
            ConnState::Connected if pending > 0 => "#aa8800",
            ConnState::Connected => "#227722",
            ConnState::TwoTabs => "#444",
        }
    };
    let label = move || {
        let state = CONN_STATE.get();
        let pending = PENDING_COUNT.get();
        match state {
            ConnState::Offline => "Offline".to_string(),
            ConnState::Connecting => "Connecting…".to_string(),
            ConnState::FailedToConnect => "Can't reach server".to_string(),
            ConnState::Connected if pending > 0 => format!("Syncing ({pending})"),
            ConnState::Connected => "Connected".to_string(),
            ConnState::TwoTabs => "Another tab is connected".to_string(),
        }
    };
    view! {
        <div
            class="connection-indicator"
            style=move || {
                format!(
                    "background-color: {}; color: white; padding: 4px 8px; font-size: 12px;",
                    bg()
                )
            }
        >
            {label}
        </div>
    }
}