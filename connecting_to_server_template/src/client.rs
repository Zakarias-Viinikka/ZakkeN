pub mod border;
pub mod panel;
pub mod server_messages;
pub mod status_line;

use crate::client::border::{current_border_state, sync_work_in_flight, BorderComponent};
use crate::client::server_messages::spawn_receive_loop;
use crate::client_server_coms::{ComsHandleForClient, MessageToClient};
use futures::channel::mpsc::UnboundedReceiver;
use leptos::prelude::*;
use std::sync::LazyLock;

static CLIENT_STARTED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub fn client_started_signal() -> ArcRwSignal<bool> {
    CLIENT_STARTED.clone()
}

#[component]
pub fn Client(
    handle: ComsHandleForClient,
    rx: UnboundedReceiver<MessageToClient>,
    children: ChildrenFn,
) -> impl IntoView {
    provide_context(handle.clone());
    spawn_receive_loop(handle, rx);
    view! {
        <BorderComponent state=current_border_state() in_flight=sync_work_in_flight()>
            {children()}
        </BorderComponent>
    }
}