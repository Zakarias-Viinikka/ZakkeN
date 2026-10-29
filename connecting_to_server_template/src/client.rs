use crate::client_server_coms::{ComsHandleForClient, MessageToClient};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

#[component]
fn BorderComponent(
    #[prop(into)] online: Signal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = move || {
        if online.get() {
            "border border-online"
        } else {
            "border"
        }
    };
    view! {
        <div class=class>
            {children()}
        </div>
    }
}

pub struct ServerMessageReceiver {
    #[allow(dead_code)]
    handle: ComsHandleForClient,
}

impl ServerMessageReceiver {
    fn handle(&self, msg: MessageToClient) {
        match msg {
            MessageToClient::AllMyData => {
                // TODO
            }
        }
    }
}

#[component]
pub fn Client(
    handle: ComsHandleForClient,
    rx: UnboundedReceiver<MessageToClient>,
    children: ChildrenFn,
) -> impl IntoView {
    let online = RwSignal::new(false);
    let receiver = Rc::new(ServerMessageReceiver { handle: handle.clone() });
    let receiver_loop = receiver.clone();
    let mut rx = rx;
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver_loop.handle(msg);
        }
    });
    view! {
        <BorderComponent online=online>
            {children()}
        </BorderComponent>
    }
}