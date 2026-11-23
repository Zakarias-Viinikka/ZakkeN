use std::sync::LazyLock;

use leptos::prelude::*;

pub static APP_LOCKED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));