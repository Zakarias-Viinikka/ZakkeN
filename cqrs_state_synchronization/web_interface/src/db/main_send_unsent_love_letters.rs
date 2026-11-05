use std::sync::{LazyLock, Mutex};

use futures_channel::mpsc::UnboundedSender;

static KICK_TX: LazyLock<Mutex<Option<UnboundedSender<()>>>> = LazyLock::new(|| Mutex::new(None));

pub fn main_send_unsent_love_letters() {
    if !crate::owner_of_interacting_with_server::sync_may_run() {
        return;
    }
    if let Some(tx) = KICK_TX.lock().unwrap().as_ref() {
        let _ = tx.unbounded_send(());
    }
}

pub fn install_kick_sender(sender: UnboundedSender<()>) {
    *KICK_TX.lock().unwrap() = Some(sender);
}
