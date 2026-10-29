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
    Connecting,
    Failed,
    Connected,
    NeedsReconcile,
    Lonewolf,
}

#[component]
fn BorderComponent(
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

pub struct ServerMessageReceiver {
    handle: ComsHandleForClient,
}

impl ServerMessageReceiver {
    fn handle(&self, msg: MessageToClient) {
        match msg {
            MessageToClient::LoveLetterFromServer(letter) => {
                crate::server_link::receive_letter(&self.handle, letter);
            }
            MessageToClient::ConnectionLost => {
                if crate::server_link::restart_in_flight_signal().get_untracked() {
                    return;
                }
                let was_connected = crate::server_link::conn_state_signal().get_untracked()
                    == crate::server_link::ConnState::Connected;
                crate::server_link::conn_state_signal().set(crate::server_link::ConnState::Offline);
                crate::server_link::sending_signal().set(false);
                if was_connected {
                    crate::cool_popup::show_popup_as(
                        crate::cool_popup::PopupKind::Warning,
                        "Connection to the server was lost",
                    );
                }
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
        use crate::server_link::ConnState;
        use crate::sync_mode::SyncMode;
        let mode = crate::sync_mode::sync_mode_signal().get();
        let conn = crate::server_link::conn_state_signal().get();
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
    });
    let in_flight = Signal::derive(move || {
        let conn = crate::server_link::conn_state_signal().get();
        let sending = crate::server_link::sending_signal().get();
        let restart = crate::server_link::restart_in_flight_signal().get();
        conn == crate::server_link::ConnState::Connecting || sending || restart
    });
    view! {
        <BorderComponent state=state in_flight=in_flight>
            {children()}
        </BorderComponent>
    }
}

fn reason_label(reason: Option<crate::sync_mode::ModeReason>) -> &'static str {
    use crate::sync_mode::ModeReason::*;
    match reason {
        Some(YouPaused) => "you paused",
        Some(YouWiped) => "you wiped",
        Some(ServerChanged) => "server changed",
        Some(WipeIncomplete) => "wipe incomplete",
        Some(AdoptIncomplete) => "adopt incomplete",
        Some(Reconciling) => "reconciling",
        Some(ReconcileIncomplete) => "reconcile incomplete",
        None => "unknown",
    }
}

fn status_line() -> Signal<String> {
    Signal::derive(move || {
        use crate::server_link::ConnState;
        use crate::sync_mode::SyncMode;
        let mode = crate::sync_mode::sync_mode_signal().get();
        let conn = crate::server_link::conn_state_signal().get();
        let sending = crate::server_link::sending_signal().get();
        let last_failed = crate::server_link::last_send_failed_signal().get();
        let pending = crate::server_link::pending_count_signal().get();
        let plan_open = crate::reconcile::plan_signal().get().is_some();
        let plan_error = crate::reconcile::error_signal().get().is_some();
        let reason = crate::sync_mode::mode_reason_signal().get();

        if plan_open {
            let suffix = if plan_error { " Last apply failed." } else { "" };
            return format!("Reconcile in progress, sync stopped.{suffix}");
        }
        let connected = if conn == ConnState::Connected { "(connected)" } else { "(offline)" };
        match mode {
            SyncMode::NeedsReconcile => {
                let presence = if conn == ConnState::Connected { "Lonewolf" } else { "Stopped" };
                format!("{presence}: {}. Reconcile or Adopt. {connected}", reason_label(reason))
            }
            SyncMode::Connected => match conn {
                ConnState::Connecting => "Connecting…".into(),
                ConnState::FailedToConnect => format!("Couldn't connect. {pending} waiting."),
                ConnState::Offline => format!("Not connected. {pending} edits waiting."),
                ConnState::Connected => {
                    if sending {
                        format!("Sending {pending}…")
                    } else if last_failed {
                        format!("{pending} waiting: last send failed.")
                    } else if pending == 0 {
                        "In sync.".into()
                    } else {
                        format!("{pending} edits waiting.")
                    }
                }
            },
        }
    })
}

#[component]
pub fn ClientPanel() -> impl IntoView {
    let handle = expect_context::<ComsHandleForClient>();
    let conn = crate::server_link::conn_state_signal();
    let mode = crate::sync_mode::sync_mode_signal();
    let reason = crate::sync_mode::mode_reason_signal();
    let pending = crate::server_link::pending_count_signal();
    let applied = crate::server_link::applied_count_signal();
    let ignored = crate::server_link::ignored_while_stopped_signal();
    let curr_gen = crate::server_link::current_generation_signal();
    let stored_gen = crate::server_link::stored_generation_signal();
    let status = status_line();
    let handle_a = handle.clone();
    let handle_b = handle;

    let curr_a = curr_gen.clone();
    let stored_a = stored_gen.clone();
    let gen_mismatch = Signal::derive(move || {
        let c = curr_a.get();
        let s = stored_a.get();
        matches!((c, s), (Some(c), Some(s)) if c != s)
    });
    let gm1 = gen_mismatch.clone();
    let gm2 = gen_mismatch.clone();
    let curr_b = curr_gen.clone();
    let stored_b = stored_gen.clone();

    view! {
        <p style="font-weight:600; margin-bottom:0.75rem;">{move || status.get()}</p>
        <p>"mode: " {move || format!("{:?}", mode.get())}
            {move || reason.get().map(|r| format!(" ({})", reason_label(Some(r))))}
        </p>
        <p>"conn: " {move || format!("{:?}", conn.get())}</p>
        <p>"pending: " {move || pending.get()}</p>
        <p>"applied: " {move || applied.get()}</p>
        <p>"ignored while stopped: " {move || ignored.get()}</p>
        <p style=move || if gm1.get() { "color:#e8b060;" } else { "" }>
            "login gen: " {move || format!("{:?}", curr_b.get())}
        </p>
        <p style=move || if gm2.get() { "color:#e8b060;" } else { "" }>
            "stored gen: " {move || format!("{:?}", stored_b.get())}
        </p>
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