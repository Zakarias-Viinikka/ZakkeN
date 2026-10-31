use crate::client::components::border::BorderComponent;
use crate::client::server_messages::spawn_receive_loop;
use crate::client_server_coms::{ComsHandleForClient, MessageToClient};
use futures::channel::mpsc::UnboundedReceiver;
use leptos::prelude::*;

#[component]
pub fn Client(
    handle: ComsHandleForClient,
    rx: UnboundedReceiver<MessageToClient>,
    children: ChildrenFn,
) -> impl IntoView {
    provide_context(handle.clone());
    spawn_receive_loop(handle, rx);
    view! {
        <BorderComponent>
            {children()}
        </BorderComponent>
    }
}

