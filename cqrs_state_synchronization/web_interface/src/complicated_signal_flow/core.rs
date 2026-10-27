//! Signal flow for pages.

use leptos::prelude::*;

use crate::page_state::Page;

/// A block, as the whole app stores and passes it around.
#[derive(Clone, Debug)]
pub struct BlockRow {
    pub block_id: String,
    pub page_id: String,
    pub content: String,
    pub is_title: bool,
}


/// One row in the nav menu's page list.
#[derive(Clone)]
pub struct LocalPages {
    pub db_id: usize,
    pub title: String,
    pub block_id: String,
    pub page_id: String,
    pub is_disabled: bool,
}

// ---------- batcher ----------

use std::sync::Arc;
use std::sync::LazyLock;

use error_stuff::CqrsErr;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_channel::oneshot;
use futures_util::StreamExt;
use protocol::error::DbError;
use my_yrs_lib::BossOfYrs;
use text_diff::diff_batching::combine_list_of_getdiff_results::combine_getdiff_results;
use text_diff::diff_logic::DiffResult;

use crate::db::ui_actions;

/// Set to true when a flush fails and the on-screen state can no longer
/// be trusted. Nothing stops the user from continuing to type when this
/// is true. When it flips to true, the batcher shows a popup asking the
/// user to copy what they need and refresh, and `MainContent` changes
/// the background color. The next flush will fail the same way, and
/// this stays true.
pub static APP_LOCKED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

/// One queued write. The queue is ordered: an `AddBlock` must reach
/// yrs before any edit to the block it creates.
enum PendingEdit {
    Edit {
        boss: Arc<BossOfYrs>,
        block_id: String,
        diff: DiffResult,
    },
    AddBlock {
        boss: Arc<BossOfYrs>,
        block_id: String,
        content: String,
        position: my_yrs_lib::yrs_wrapper::PositionToInsert,
        is_part_of_main_menu_page: bool,
        session_id: String,
    },
}

enum BatcherMsg {
    Poke,
    Flush { reply: oneshot::Sender<Result<(), CqrsErr>> },
}

/// So the way to talk to a batcher isn't mixed up with actual
/// batcher logic. rx/tx is annoying with leptos; signals when
/// possible.
#[derive(Clone)]
pub struct BatcherHandle {
    pending: RwSignal<Vec<PendingEdit>>,
    sender: UnboundedSender<BatcherMsg>,
}

impl BatcherHandle {
    pub fn push_edit(&self, boss: Arc<BossOfYrs>, block_id: String, diff: DiffResult) {
        self.pending
            .update(|list| list.push(PendingEdit::Edit { boss, block_id, diff }));
        let _ = self.sender.unbounded_send(BatcherMsg::Poke);
    }

    /// Queue the creation of a block whose id the UI already chose.
    /// Returns immediately. The add runs at flush time, in order with
    /// the edits around it.
    pub fn push_add_block(
        &self,
        boss: Arc<BossOfYrs>,
        block_id: String,
        content: String,
        position: my_yrs_lib::yrs_wrapper::PositionToInsert,
        is_part_of_main_menu_page: bool,
        session_id: String,
    ) {
        self.pending.update(|list| {
            list.push(PendingEdit::AddBlock {
                boss,
                block_id,
                content,
                position,
                is_part_of_main_menu_page,
                session_id,
            })
        });
        let _ = self.sender.unbounded_send(BatcherMsg::Poke);
    }

    pub async fn flush_now(&self) -> Result<(), CqrsErr> {
        let (reply, response) = oneshot::channel();
        if self
            .sender
            .unbounded_send(BatcherMsg::Flush { reply })
            .is_err()
        {
            return Err(CqrsErr::DbErrorContainer(DbError::BadCode(
                "batcher run loop is gone".to_string(),
            )));
        }
        match response.await {
            Ok(result) => result,
            Err(_canceled) => Err(CqrsErr::DbErrorContainer(DbError::BadCode(
                "batcher dropped the flush reply".to_string(),
            ))),
        }
    }
}

/// Batches text edits so every keyboard input isn't an edit. Also
/// supports adding undo/redo support more cleanly.
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

pub struct Batcher {
    pending: RwSignal<Vec<PendingEdit>>,
    receiver: UnboundedReceiver<BatcherMsg>,
}

impl Batcher {
    pub fn new() -> (Self, BatcherHandle) {
        let pending = RwSignal::new(Vec::new());
        let (sender, receiver) = unbounded();
        (
            Batcher { pending, receiver },
            BatcherHandle { pending, sender },
        )
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.next().await {
            match msg {
                BatcherMsg::Poke => self.debounce_then_flush().await,
                BatcherMsg::Flush { reply } => {
                    let result = self.flush().await;
                    let _ = reply.send(result);
                }
            }
        }
    }

    async fn debounce_then_flush(&mut self) {
        loop {
            let timeout = gloo_timers::future::TimeoutFuture::new(500);
            futures_util::pin_mut!(timeout);

            let msg = {
                let next = self.receiver.next();
                futures_util::pin_mut!(next);
                match futures_util::future::select(timeout, next).await {
                    futures_util::future::Either::Left(_) => None,
                    futures_util::future::Either::Right((m, _)) => m,
                }
            };

            match msg {
                None => {
                    let _ = self.flush().await;
                    return;
                }
                Some(BatcherMsg::Poke) => continue,
                Some(BatcherMsg::Flush { reply }) => {
                    let result = self.flush().await;
                    let _ = reply.send(result);
                    return;
                }
            }
        }
    }

    async fn flush(&mut self) -> Result<(), CqrsErr> {
        let mut pending: Vec<PendingEdit> = Vec::new();
        self.pending.update(|list| pending = std::mem::take(list));
        if pending.is_empty() {
            return Ok(());
        }

        // One writer at a time. Everything below — the transaction, the
        // executor calls, the commit — runs under this guard. Dropped at
        // the end of the function, including on any early error return.
        let _write_guard = crate::db::write_lock::WRITE_LOCK.acquire().await;

        let session_id = crate::db::cached_ids::SESSION_ID
            .get_untracked()
            .expect(crate::db::cached_ids::INIT_ERR);

        web_internal_db::db_helper::begin_all_or_nothing().await?;

        // Edits are grouped per block until an add shows up. The group
        // is written first so the add lands after every edit queued
        // before it, then grouping starts over.
        let mut failure: Option<CqrsErr> = None;
        let mut run: Vec<(String, Arc<BossOfYrs>, Vec<DiffResult>)> = Vec::new();
        for op in pending {
            match op {
                PendingEdit::Edit { boss, block_id, diff } => {
                    if let Some(existing) = run.iter_mut().find(|(bid, _, _)| bid == &block_id) {
                        existing.2.push(diff);
                    } else {
                        run.push((block_id, boss, vec![diff]));
                    }
                }
                PendingEdit::AddBlock {
                    boss,
                    block_id,
                    content,
                    position,
                    is_part_of_main_menu_page,
                    session_id: add_session_id,
                } => {
                    if let Err(e) = write_edit_run(std::mem::take(&mut run), &session_id).await {
                        failure = Some(e);
                        break;
                    }
                    if let Err(e) = ui_actions::add_block_unflushed(
                        boss,
                        block_id,
                        content,
                        String::new(),
                        position,
                        is_part_of_main_menu_page,
                        add_session_id,
                    )
                    .await
                    {
                        failure = Some(e);
                        break;
                    }
                }
            }
        }
        if failure.is_none() {
            if let Err(e) = write_edit_run(run, &session_id).await {
                failure = Some(e);
            }
        }

        match failure {
            None => {
                web_internal_db::db_helper::everything_went_perfectly().await?;
                crate::server_link::kick_send();
                Ok(())
            }
            Some(e) => {
                let _ = web_internal_db::db_helper::regret_everything().await;
                if !APP_LOCKED.get_untracked() {
                    APP_LOCKED.set(true);
                    crate::leptos_components::small_components::popup::create_popup(
                        "Data corrupted. Copy any recent changes you need, then refresh the page."
                            .to_string(),
                    );
                }
                Err(e)
            }
        }
    }
}

async fn write_edit_run(
    run: Vec<(String, Arc<BossOfYrs>, Vec<DiffResult>)>,
    session_id: &str,
) -> Result<(), CqrsErr> {
    for (block_id, boss, diffs) in run {
        for diff in combine_getdiff_results(diffs) {
            ui_actions::edit_block(boss.clone(), block_id.clone(), diff, session_id.to_string())
                .await?;
        }
    }
    Ok(())
}

/// One keystroke's worth of information.
///
/// Users:
/// - MyBlockContent: the component that represents a textblock that is in DragReorder.
/// - on_block_input: the callback that gets run when the text changes in the textblock component.
/// - check_if_should_make_new_block: receives it so that if the conditions are met, the batcher is told to flush, then a new block is made and the cursor moves there.
pub struct BlockInputEvent {
    pub block_id: String,
    pub old_text: String,
    pub new_text: String,
    pub was_enter: bool,
    /// used for telling if the latest edit was at end of block
    pub cursor_pos: usize,
}

/// The pages currently open. Held globally so the yrs docs and block
/// lists survive route changes. MainContent reads from here; it no
/// longer creates its own.
///
/// page1 is the page the viewer is showing. page2 is for the two-page
/// view, which isn't built yet - it's always None today.
#[derive(Clone, Copy)]
pub struct OpenPages {
    pub page1: Page,
    pub page2: RwSignal<Option<Page>>,
}

/// Whoever sets this to `Some(block_id)` asks that block's textarea
/// to grab focus, with the cursor at the end. `MyBlockContent`
/// watches it and clears it after focusing, so each request fires
/// exactly once.
pub static FOCUS_BLOCK_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

/// Same as `FOCUS_BLOCK_ID` but parks the cursor at the start of the
/// block instead of the end. Used after a split, where the block's
/// content is what used to be after the cursor.
pub static FOCUS_BLOCK_AT_START: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));
