use crate::data_ops::verb_error::VerbError;
use leptos::prelude::*;
use std::sync::LazyLock;

static DATASET_OP_BUSY: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub fn dataset_op_busy() -> ArcRwSignal<Option<String>> {
    DATASET_OP_BUSY.clone()
}

pub async fn with_busy<F, T>(message: impl Into<String>, fut: F) -> Result<T, VerbError>
where
    F: std::future::Future<Output = Result<T, VerbError>>,
{
    if DATASET_OP_BUSY.get_untracked().is_some() {
        return Err(VerbError::refused("another operation is running"));
    }
    DATASET_OP_BUSY.set(Some(message.into()));
    let result = fut.await;
    DATASET_OP_BUSY.set(None);
    result
}
