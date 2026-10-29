use crate::client_server_coms::{ComsHandleForServer, MessageToServer};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

pub struct ClientMessageReceiver {
    pub server_online: RwSignal<bool>,
    handle: ComsHandleForServer,
}

impl ClientMessageReceiver {
    fn handle(&self, msg: MessageToServer) {
        match msg {
            MessageToServer::AreYouOnline(tx) => {
                let _ = tx.send(self.server_online.get());
            }
            MessageToServer::WipeYourself => {
                // TODO
            }
            MessageToServer::GiveMeAllYourData => {
                self.handle.send_all_my_data();
            }
            MessageToServer::StartOverFromMyData => {
                // TODO
            }
        }
    }
}

#[component]
pub fn Server(
    handle: ComsHandleForServer,
    rx: UnboundedReceiver<MessageToServer>,
    children: ChildrenFn,
) -> impl IntoView {
    let server_online = RwSignal::new(true);
    let receiver = Rc::new(ClientMessageReceiver {
        server_online,
        handle: handle.clone(),
    });
    let receiver_loop = receiver.clone();
    let mut rx = rx;
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver_loop.handle(msg);
        }
    });
    view! {
        <BorderComponent online=server_online>
            {children()}
        </BorderComponent>
    }
}

#[component]
fn BorderComponent(#[prop(into)] online: Signal<bool>, children: ChildrenFn) -> impl IntoView {
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
