use std::sync::LazyLock;

use leptos::prelude::*;

pub static APP_LOCKED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub static FOCUS_BLOCK_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static FOCUS_BLOCK_AT_START: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));
