use leptos::prelude::*;

use crate::shared_owners::client_status::{client_status_signal, ClientStatus};

#[component]
pub fn BorderComponent(children: ChildrenFn) -> impl IntoView {
    let status = client_status_signal();
    let in_flight = sync_work_in_flight();
    let class = move || {
        let base = match status.get() {
            ClientStatus::Connected => "border border-online",
            ClientStatus::NeedsReconcile => "border border-needs-reconcile",
            ClientStatus::Lonewolf => "border border-lonewolf",
            ClientStatus::Locked | ClientStatus::FailedToConnect => "border border-failed",
            ClientStatus::NotStarted | ClientStatus::Offline | ClientStatus::Connecting => "border",
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

pub fn sync_work_in_flight() -> Signal<bool> {
    Signal::derive(move || {
        use crate::owner_of_interacting_with_server::conn_state::ConnState;
        let conn = crate::owner_of_interacting_with_server::conn_state::conn_state_signal().get();
        let sending = crate::owner_of_interacting_with_server::sending::sending_signal().get();
        let restart = crate::owner_of_interacting_with_server::restart::restart_in_flight_signal().get();
        conn == ConnState::Connecting || sending || restart
    })
}