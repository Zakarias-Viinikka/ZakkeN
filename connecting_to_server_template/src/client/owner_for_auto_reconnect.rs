//! Owner for "the socket is down, keep trying to bring it back".
//!
//! Watches `client_status_signal`. When it lands on Offline or
//! FailedToConnect, starts a loop that pops "Trying to reconnect…",
//! calls `restart_sync_stack`, and either exits (some outcome other
//! than Failed — the caller was told what happened) or waits 5s and
//! tries again. The loop breaks by itself the moment the client status
//! stops being offline; no external cancellation is needed.
//!
//! Caller pov: call `spawn_owner_for_auto_reconnect(client_handle)`
//! once, in `AppMain`. Done.

use crate::client_server_coms::ComsHandleForClient;
use crate::shared_owners::client_status::{client_status_signal, ClientStatus};
use leptos::prelude::*;

pub fn main_for_retrying_the_connection_while_offline(client_handle: ComsHandleForClient) {
    let retry_running = RwSignal::new(false);
    Effect::new(move |_| {
        let running = retry_running.get();
        let status = client_status_signal().get();
        let should_retry = matches!(
            status,
            ClientStatus::Offline | ClientStatus::FailedToConnect
        );
        if !should_retry || running {
            return;
        }
        retry_running.set(true);
        let client_handle = client_handle.clone();
        let retry_running_for_loop = retry_running.clone();
        leptos::task::spawn_local(async move {
            loop {
                {
                    let (tx, rx) = futures::channel::oneshot::channel::<()>();
                    leptos::prelude::set_timeout(
                        move || {
                            let _ = tx.send(());
                        },
                        std::time::Duration::from_millis(5000),
                    );
                    let _ = rx.await;
                }
                let status = client_status_signal().get_untracked();
                if !matches!(
                    status,
                    ClientStatus::Offline | ClientStatus::FailedToConnect
                ) {
                    break;
                }
                crate::cool_popup::show_popup_as(
                    crate::cool_popup::PopupKind::Warning,
                    "Trying to reconnect…",
                );
                let outcome = crate::owner_of_interacting_with_server::restart::restart_sync_stack(
                    client_handle.clone(),
                )
                .await;
                match outcome {
                    crate::owner_of_interacting_with_server::restart::RestartOutcome::Failed(_) => {
                        // stay in the loop, retry again in 5s
                    }
                    other => {
                        // JustChanged pops "Server dataset changed. Sync is
                        // stopped until you Reconcile or Adopt." — which is
                        // exactly the case where the client was offline when
                        // another client wiped the server, and the retry
                        // reconnect is the first time it hears about it.
                        crate::ui_ops::announce_connect_outcome(other);
                        break;
                    }
                }
            }
            retry_running_for_loop.set(false);
        });
    });
}
