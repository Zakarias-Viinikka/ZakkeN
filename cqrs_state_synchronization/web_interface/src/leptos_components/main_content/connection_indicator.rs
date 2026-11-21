use drift_guard_client::connection_status::get_conn_color;
use leptos::prelude::*;

use crate::owner_of_interacting_with_server::{ConnState, PENDING_COUNT};
use crate::plexget_conn_state;

#[component]
pub fn ConnectionIndicator() -> impl IntoView {
    let bg = move || {
        let state = plexget_conn_state();
        //let pending = PENDING_COUNT.get();
        get_conn_color(state)
    };
    let label = move || {
        let state = plexget_conn_state();
        let pending = PENDING_COUNT.get();
        match state {
            ConnState::Offline => "Offline".to_string(),
            ConnState::Connecting(_) => "Connecting…".to_string(),
            ConnState::LoggingIn => "Logging in…".to_string(),
            ConnState::FailedToConnect => "Can't reach server".to_string(),
            ConnState::Connected if pending > 0 => format!("Syncing ({pending})"),
            ConnState::Connected => "Connected".to_string(),
            ConnState::TwoTabs => "Another tab is connected".to_string(),
            ConnState::Lonewolf => "Sync stopped".to_string(),
            ConnState::LonewolfAndOffline => "Offline, sync stopped".to_string(),
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
