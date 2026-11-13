//! Caller pov: the `/reconcile` route. Loads the state, then hands off
//! to one of the sub-views based on the phase.
//!
//! The Effect tracks CONN_STATE via `.get()` so it re-fires as the state
//! settles after a reconnect. A `load_started` flag makes the load run
//! only once per mount even if the state flutters.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::owner_of_interacting_with_server::ConnState;
use crate::plexget_conn_state;

use super::load_and_classify::load_and_classify;
use super::not_lonewolf_view::NotLonewolfView;
use super::phase::Phase;
use super::ready_view::ReadyView;


#[component]
pub fn ReconcilePage() -> impl IntoView {
    let phase = RwSignal::new(Phase::Loading);
    let load_started = RwSignal::new(false);

    Effect::new(move |_| {
        let current = plexget_conn_state();
        match current {
            ConnState::Lonewolf | ConnState::LonewolfAndOffline => {
                if !load_started.get_untracked() {
                    load_started.set(true);
                    phase.set(Phase::Loading);
                    spawn_local(async move {
                        phase.set(match load_and_classify().await {
                            Ok(p) => p,
                            Err(e) => Phase::Error(e),
                        });
                    });
                }
            }
            other => {
                phase.set(Phase::NotLonewolf(other));
            }
        }
    });

    view! {
        <div class="rp-page">
            <div class="rp-page-inner">
                <div class="rp-header">
                    <h1>"Reconcile"</h1>
                    <span class="rp-header-sub">
                        "resolve pages that differ between this client and the server"
                    </span>
                </div>
                {move || match phase.get() {
                    Phase::Loading => view! {
                        <div class="rp-loading">"Reading pages and asking the server..."</div>
                    }.into_any(),
                    Phase::NotLonewolf(s) => view! {
                        <NotLonewolfView state=s />
                    }.into_any(),
                    Phase::Error(e) => view! {
                        <div class="rp-error">{format!("error: {e}")}</div>
                    }.into_any(),
                    Phase::Ready {
                        target_drift_version,
                        target_reconcile_version,
                        mismatched,
                        client_only,
                        mount_fingerprints,
                        user_id,
                        time,
                    } => view! {
                        <ReadyView
                            target_drift_version=target_drift_version
                            target_reconcile_version=target_reconcile_version
                            mismatched=mismatched
                            client_only=client_only
                            mount_fingerprints=mount_fingerprints
                            user_id=user_id
                            time=time
                        />
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
