use error_stuff::CqrsErr;

use super::handle::BatcherHandle;

/// Run `work` as the only writer: flush the batcher, then take the
/// write lock, then run `work`, then release. Used by the receive
/// worker so a remote apply can't interleave with a local flush.
///
/// Deadlock rule: nothing inside `work` may call `flush_now()`. The
/// lock is held by then, and `flush_now` takes it again.
pub async fn run_exclusive<Fut, F, T>(
    batcher: &BatcherHandle,
    work: F,
) -> Result<T, CqrsErr>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, CqrsErr>>,
{
    batcher.flush_now().await?;
    let guard = crate::db::write_lock::WRITE_LOCK.acquire().await;
    let result = work().await;
    drop(guard);
    result
}
