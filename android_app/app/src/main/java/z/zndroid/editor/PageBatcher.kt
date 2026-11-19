package z.zndroid.editor

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import rustlib.data_builder_for_operations_that_need_to_be_correct.addBlock
import rustlib.data_builder_for_operations_that_need_to_be_correct.deleteBlock
import rustlib.data_builder_for_operations_that_need_to_be_correct.editBlock
import rustlib.data_builder_for_operations_that_need_to_be_correct.moveBlock
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.text_diff.DiffResult
import rustlib.text_diff.combineGetdiffResults
import z.zndroid.DbManager
import z.zndroid.Storage.SessionManager
import z.zndroid.WriteLock
import z.zndroid.actions.kickSend
import z.zndroid.components.GlobalPopupManager
import z.zndroid.executor.addBlockRequiresThreeDbActions
import z.zndroid.executor.deleteBlockRequiresThreeDbActions
import z.zndroid.executor.editBlockRequiresThreeDbInserts
import z.zndroid.executor.moveBlockRequiresThreeDbActions

class PageBatcher(
    private val boss: BossOfYrs,
    private val pageId: String,
    private val isMainMenu: Boolean,
    private val onFailed: () -> Unit
) {
    private sealed class Entry {
        data class Edit(val blockId: String, val diff: DiffResult) : Entry()
        data class Add(val blockId: String, val content: String, val position: PositionToInsert) : Entry()
        data class Delete(val blockId: String) : Entry()
        data class Move(val blockId: String, val above: String?, val below: String?) : Entry()
    }

    private val queue = mutableListOf<Entry>()
    private val queueLock = Any()
    private val flushMutex = Mutex()

    // Guards [debounceJob]. Every read and write of the field happens
    // inside synchronized(debounceLock) so armDebounce / flushNow /
    // destroy can't race each other. See STORED context (2026-11-19).
    private val debounceLock = Any()
    private var debounceJob: Job? = null

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // Set by [destroy]. Once true, pushes are dropped and no further flush
    // runs. This stops a batcher whose session has been replaced from
    // flushing edits against a destroyed boss.
    @Volatile
    private var destroyed = false

    @Volatile
    private var closing = false

    fun pushEdit(blockId: String, diff: DiffResult) {
        if (destroyed) return
        synchronized(queueLock) { queue.add(Entry.Edit(blockId, diff)) }
        armDebounce()
    }

    fun pushAdd(blockId: String, content: String, position: PositionToInsert) {
        if (destroyed) return
        synchronized(queueLock) { queue.add(Entry.Add(blockId, content, position)) }
        armDebounce()
    }

    fun pushDelete(blockId: String) {
        if (destroyed) return
        synchronized(queueLock) { queue.add(Entry.Delete(blockId)) }
        armDebounce()
    }

    fun pushMove(blockId: String, above: String?, below: String?) {
        if (destroyed) return
        synchronized(queueLock) { queue.add(Entry.Move(blockId, above, below)) }
        armDebounce()
    }

    /**
     * Arm (or re-arm) the 500 ms debounce. Only the sleep runs on
     * [debounceJob]; the actual flush is launched on a *separate*
     * coroutine that [debounceJob] cannot cancel. This mirrors the web
     * batcher, where the run loop never cancels an in-flight flush.
     * Without the split, a fresh push arriving while a flush was inside
     * its transaction would cancel it and lose the already-drained batch.
     */
    private fun armDebounce() {
        if (destroyed) return
        synchronized(debounceLock) {
            debounceJob?.cancel()
            debounceJob = scope.launch {
                delay(500)
                if (destroyed) return@launch
                scope.launch { runOneFlush() }
            }
        }
    }

    /**
     * Permanently stops this batcher. Pending queued entries are dropped;
     * callers must flush first if they want them written. The scope is
     * cancelled so no later debounce can fire.
     */
    fun destroy() {
        destroyed = true
        synchronized(debounceLock) {
            debounceJob?.cancel()
            debounceJob = null
        }
        synchronized(queueLock) { queue.clear() }
        scope.cancel()
    }

    suspend fun flushNow(): Result<Unit> {
        synchronized(debounceLock) {
            debounceJob?.cancel()
            debounceJob = null
        }
        return runOneFlush()
    }

    suspend fun flush(): Result<Unit> = flushNow()

    /** True while edits are queued or a flush is writing. */
    fun hasPending(): Boolean =
        flushMutex.isLocked || synchronized(queueLock) { queue.isNotEmpty() }

    /**
     * Flush in the background. [onSuccess] runs on Main only if the flush
     * succeeded. On failure [onFailed] has already fired (inside
     * [runOneFlush]) - do not double-fire it here.
     */
    fun flushThen(onSuccess: () -> Unit) {
        if (destroyed) return
        scope.launch {
            val result = runOneFlush()
            if (result.isSuccess) {
                withContext(Dispatchers.Main) { onSuccess() }
            }
        }
    }

    fun flushAsync() {
        if (destroyed) return
        scope.launch { runOneFlush() }
    }

    /**
     * Marks this batcher as closing, flushes any pending entries, invokes
     * [onFlushed] (e.g. to destroy the yrs boss) even if the flush fails,
     * and permanently destroys the batcher.
     */
    fun flushAndDestroyAsync(onFlushed: () -> Unit): Job {
        closing = true
        synchronized(debounceLock) {
            debounceJob?.cancel()
            debounceJob = null
        }
        return scope.launch {
            try {
                runOneFlush()
            } finally {
                onFlushed()
                destroy()
            }
        }
    }

    private suspend fun runOneFlush(): Result<Unit> = flushMutex.withLock {
        if (destroyed) return Result.success(Unit)
        val batch: List<Entry> = synchronized(queueLock) {
            if (queue.isEmpty()) return Result.success(Unit)
            val b = queue.toList()
            queue.clear()
            b
        }
        val result = runCatching {
            WriteLock.withLock {
                DbManager.withTransaction {
                    writeBatch(batch)
                }.getOrThrow()
            }
        }
        result.fold(
            onSuccess = {
                kickSend()
                Result.success(Unit)
            },
            onFailure = { e ->
                if (e is CancellationException) {
                    Result.failure(e)
                } else {
                    // Popup and onFailed (which reloads the page) both
                    // mutate UI state; dispatch to Main. When closing,
                    // the page is already leaving so don't trigger a reload.
                    withContext(Dispatchers.Main) {
                        if (closing) {
                            GlobalPopupManager.showError("Save failed: ${e.message}")
                        } else {
                            GlobalPopupManager.showError("Save failed: ${e.message}. Reloading page.")
                            onFailed()
                        }
                    }
                    Result.failure(e)
                }
            }
        )
    }

    private suspend fun writeBatch(batch: List<Entry>) {
        val sessionId = SessionManager.currentSessionId
        val editRun = mutableListOf<Pair<String, DiffResult>>()

        suspend fun flushEditRun() {
            if (editRun.isEmpty()) return
            val grouped = editRun.groupBy({ it.first }, { it.second })
            for ((blockId, diffs) in grouped) {
                val squashed = combineGetdiffResults(diffs)
                for (diff in squashed) {
                    val ctx = editBlock(boss, blockId, diff, sessionId) ?: continue
                    editBlockRequiresThreeDbInserts(ctx).getOrThrow()
                }
            }
            editRun.clear()
        }

        for (entry in batch) {
            when (entry) {
                is Entry.Edit -> editRun.add(entry.blockId to entry.diff)
                is Entry.Add -> {
                    flushEditRun()
                    val ctx = addBlock(
                        yrs = boss,
                        blockContent = entry.content,
                        blockMetaData = "",
                        position = entry.position,
                        isPartOfMainMenuPage = isMainMenu,
                        sessionId = sessionId,
                        insertWithExistingBlockId = entry.blockId
                    )
                    addBlockRequiresThreeDbActions(ctx).getOrThrow()
                }
                is Entry.Delete -> {
                    flushEditRun()
                    val ctx = deleteBlock(boss, entry.blockId, sessionId)
                    deleteBlockRequiresThreeDbActions(ctx).getOrThrow()
                }
                is Entry.Move -> {
                    flushEditRun()
                    val ctx = moveBlock(boss, entry.blockId, entry.above, entry.below, sessionId)
                    moveBlockRequiresThreeDbActions(ctx).getOrThrow()
                }
            }
        }
        flushEditRun()
    }
}