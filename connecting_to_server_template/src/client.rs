use crate::CustomButton;
use crate::client_server_coms::{ComsHandleForClient, MessageToClient};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BorderState {
    Offline,
    Paused,
    Connected,
}

#[component]
fn BorderComponent(
    #[prop(into)] state: Signal<BorderState>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = move || match state.get() {
        BorderState::Connected => "border border-online",
        BorderState::Paused => "border border-paused",
        BorderState::Offline => "border",
    };
    view! {
        <div class=class>
            {children()}
        </div>
    }
}

pub struct ServerMessageReceiver {
    handle: ComsHandleForClient,
}

impl ServerMessageReceiver {
    fn handle(&self, msg: MessageToClient) {
        match msg {
            MessageToClient::LoveLetterFromServer(letter) => {
                crate::server_link::receive_letter(&self.handle, letter);
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
    provide_context(handle.clone());
    let receiver = Rc::new(ServerMessageReceiver { handle: handle.clone() });
    let receiver_loop = receiver.clone();
    let mut rx = rx;
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver_loop.handle(msg);
        }
    });
    let state = Signal::derive(move || {
        let conn = crate::server_link::conn_state_signal().get();
        let mode = crate::sync_mode::sync_mode_signal().get();
        match mode {
            crate::sync_mode::SyncMode::Paused => BorderState::Paused,
            crate::sync_mode::SyncMode::Connected
                if conn == crate::server_link::ConnState::Connected =>
            {
                BorderState::Connected
            }
            _ => BorderState::Offline,
        }
    });
    view! {
        <BorderComponent state=state>
            {children()}
        </BorderComponent>
    }
}

#[component]
pub fn ClientPanel() -> impl IntoView {
    let handle = expect_context::<ComsHandleForClient>();
    let conn = crate::server_link::conn_state_signal();
    let mode = crate::sync_mode::sync_mode_signal();
    let pending = crate::server_link::pending_count_signal();
    let applied = crate::server_link::applied_count_signal();
    let curr_gen = crate::server_link::current_generation_signal();
    let stored_gen = crate::server_link::stored_generation_signal();
    let handle_a = handle.clone();
    let handle_b = handle;
    view! {
        <p>"mode: " {move || format!("{:?}", mode.get())}</p>
        <p>"conn: " {move || format!("{:?}", conn.get())}</p>
        <p>"pending: " {move || pending.get()}</p>
        <p>"applied: " {move || applied.get()}</p>
        <p>"login gen: " {move || format!("{:?}", curr_gen.get())}</p>
        <p>"stored gen: " {move || format!("{:?}", stored_gen.get())}</p>
        <CustomButton
            label="Pretend edit page_a"
            on_click=move |_| {
                crate::server_link::queue_pretend_edit(&handle_a, "page_a", "edited a");
            }
        />
        <CustomButton
            label="Pretend edit page_b"
            on_click=move |_| {
                crate::server_link::queue_pretend_edit(&handle_b, "page_b", "edited b");
            }
        />
    }
}