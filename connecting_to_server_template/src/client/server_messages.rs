use crate::client_server_coms::{ComsHandleForClient, MessageToClient};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use leptos::task::spawn_local;
use std::rc::Rc;

pub fn spawn_receive_loop(
    handle: ComsHandleForClient,
    mut rx: UnboundedReceiver<MessageToClient>,
) {
    let receiver = Rc::new(ServerMessageReceiver::new(handle));
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver.handle(msg);
        }
    });
}

struct ServerMessageReceiver {
    handle: ComsHandleForClient,
}

impl ServerMessageReceiver {
    fn new(handle: ComsHandleForClient) -> Self {
        ServerMessageReceiver { handle }
    }

    fn handle(&self, msg: MessageToClient) {
        // While internet is off, drop everything from the server. This
        // matches a real socket death: nothing arrives, anything the
        // server pushed is missed, and unacked rows are re-delivered on
        // the next successful login.
        if !crate::owner_of_interacting_with_server::internet_switch::is_internet_on() {
            return;
        }
        match msg {
            MessageToClient::LoveLetterFromServer(letter) => {
                crate::owner_of_interacting_with_server::receiving::receive_letter(&self.handle, letter);
            }
            MessageToClient::ServerSaysYouAreNowLonewolf { what_happened } => {
                self.handle_server_says_you_are_now_lonewolf(&what_happened);
            }
            MessageToClient::ConnectionLost => {
                crate::owner_of_interacting_with_server::conn_state::go_offline_after_connection_lost();
            }
        }
    }

    // The server replaced its dataset (another client wiped, or started
    // over from its own version). Don't disconnect: the socket is still
    // up. Set NeedsReconcile + ServerChanged and pop one warning. Then
    // kick a fast retry that re-registers us (the server wiped its user
    // list) so reconcile/adopt can talk to the server again. Silent on
    // success; the popup already told the user what happened.
    fn handle_server_says_you_are_now_lonewolf(&self, what_happened: &str) {
        crate::sync_mode::set_sync_mode(crate::sync_mode::SyncMode::NeedsReconcile);
        crate::sync_mode::set_mode_reason(crate::sync_mode::ModeReason::ServerChanged);
        crate::owner_of_interacting_with_server::receiving::clear_seen_everywhere();
        crate::owner_of_interacting_with_server::pending_queue::clear_pending_letters();
        crate::cool_popup::show_popup_as(
            crate::cool_popup::PopupKind::Warning,
            format!(
                "{what_happened}. You are now lonewolf — sync is stopped until you reconcile or adopt."
            ),
        );
        crate::owner_of_interacting_with_server::restart::kick_lonewolf_retry(self.handle.clone());
    }
}