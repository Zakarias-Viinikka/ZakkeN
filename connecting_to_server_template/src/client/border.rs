use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BorderState {
    Offline,
    Connecting,
    Failed,
    Connected,
    NeedsReconcile,
    Lonewolf,
}

pub fn current_border_state() -> Signal<BorderState> {
    Signal::derive(move || {
        use crate::server_link::conn_state::ConnState;
        use crate::sync_mode::SyncMode;
        let mode = crate::sync_mode::sync_mode_signal().get();
        let conn = crate::server_link::conn_state::conn_state_signal().get();
        match mode {
            SyncMode::NeedsReconcile => match conn {
                ConnState::Connected => BorderState::Lonewolf,
                _ => BorderState::NeedsReconcile,
            },
            SyncMode::Connected => match conn {
                ConnState::Connected => BorderState::Connected,
                ConnState::FailedToConnect => BorderState::Failed,
                ConnState::Connecting => BorderState::Connecting,
                ConnState::Offline => BorderState::Offline,
            },
        }
    })
}

pub fn sync_work_in_flight() -> Signal<bool> {
    Signal::derive(move || {
        use crate::server_link::conn_state::ConnState;
        let conn = crate::server_link::conn_state::conn_state_signal().get();
        let sending = crate::server_link::sending::sending_signal().get();
        let restart = crate::server_link::restart::restart_in_flight_signal().get();
        conn == ConnState::Connecting || sending || restart
    })
}

#[component]
pub fn BorderComponent(
    #[prop(into)] state: Signal<BorderState>,
    #[prop(into)] in_flight: Signal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = move || {
        let base = match state.get() {
            BorderState::Connected => "border border-online",
            BorderState::NeedsReconcile => "border border-needs-reconcile",
            BorderState::Lonewolf => "border border-lonewolf",
            BorderState::Failed => "border border-failed",
            BorderState::Offline | BorderState::Connecting => "border",
        };
        if in_flight.get() {
            format!("{} border-pulse", base)
        } else {
            base.to_string()
        }
    };
    view! {
        <div class=class>
            {children()}
        </div>
    }
}
