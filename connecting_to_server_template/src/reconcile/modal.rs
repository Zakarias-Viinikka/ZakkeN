use crate::reconcile::plan::ReconcilePlan;
use leptos::prelude::*;
use std::sync::LazyLock;

static RECONCILE_PLAN: LazyLock<ArcRwSignal<Option<ReconcilePlan>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static RECONCILE_ERROR: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub fn plan_signal() -> ArcRwSignal<Option<ReconcilePlan>> {
    RECONCILE_PLAN.clone()
}

pub fn error_signal() -> ArcRwSignal<Option<String>> {
    RECONCILE_ERROR.clone()
}

pub fn clear_modal() {
    RECONCILE_PLAN.set(None);
    RECONCILE_ERROR.set(None);
}

pub fn open_modal(plan: ReconcilePlan) {
    RECONCILE_PLAN.set(Some(plan));
    RECONCILE_ERROR.set(None);
}

pub fn apply_decisions_from_modal(
    coms: crate::client_server_coms::ComsHandleForClient,
    decisions: crate::reconcile::plan::ReconcileDecisions,
) {
    leptos::task::spawn_local(async move {
        match crate::reconcile::reconcile_apply(coms, decisions).await {
            Ok(()) => clear_modal(),
            Err(e) => RECONCILE_ERROR.set(Some(e.to_string())),
        }
    });
}

pub fn cancel_from_modal(coms: crate::client_server_coms::ComsHandleForClient) {
    leptos::task::spawn_local(async move {
        let _ = crate::reconcile::reconcile_cancel(coms).await;
        clear_modal();
    });
}
