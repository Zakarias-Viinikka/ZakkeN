use leptos::task::spawn_local;

use crate::db::key_value_storage_db_actions;
use crate::leptos_components::small_components::popup::show_popup;

use super::super::ConnState;
use crate::{plexget_conn_state_untracked, plexset_conn_state};

/// Sync is stopped because the client's drift version and the server's
/// don't match. Flips the status immediately (gating takes effect at
/// once), then reads the popup-once flag in the background and shows
/// the reason (or a generic message) only if it's still true.
pub fn enter_lonewolf(reason: Option<String>) {
    let next = match plexget_conn_state_untracked() {
        ConnState::Connected | ConnState::Lonewolf | ConnState::LoggingIn => ConnState::Lonewolf,
        _ => ConnState::LonewolfAndOffline,
    };
    plexset_conn_state(next);
    spawn_local(async move {
        let should = key_value_storage_db_actions::get_should_i_drift_popup()
            .await
            .unwrap_or(true);
        if !should {
            return;
        }
        let message = match reason {
            Some(r) => format!("out of sync with server: {r}"),
            None => "out of sync with server".to_string(),
        };
        show_popup(message);
        if let Err(e) = key_value_storage_db_actions::set_should_i_drift_popup(false).await {
            leptos::logging::warn!("set_should_i_drift_popup(false): {e:?}");
        }
    });
}
