use leptos::prelude::*;
use std::sync::LazyLock;

pub(crate) static POWERED: LazyLock<ArcRwSignal<bool>> = LazyLock::new(|| ArcRwSignal::new(false));

pub fn powered_signal() -> ArcRwSignal<bool> {
    POWERED.clone()
}
