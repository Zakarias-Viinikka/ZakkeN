package z.zndroid.editor

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
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
    private var debounceJob: Job? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    fun pushEdit(blockId: String, diff: DiffResult) {
        synchronized(queueLock) { queue.add(Entry.Edit(blockId, diff)) }
        armDebounce()
    }

    fun pushAdd(blockId: String, content: String, position: PositionToInsert) {
        synchronized(queueLock) { queue.add(Entry.Add(blockId, content, position)) }
        armDebounce()
    }

    fun pushDelete(blockId: String) {
        synchronized(queueLock) { queue.add(Entry.Delete(blockId)) }
        armDebounce()
    }

    fun pushMove(blockId: String, above: String?, below: String?) {
        synchronized(queueLock) { queue.add(Entry.Move(blockId, above, below)) }
        armDebounce()
    }

    private fun armDebounce() {
        debounceJob?.cancel()
        debounceJob = scope.launch {
            delay(500)
            debounceJob = null
            runOneFlush()
        }
    }

    suspend fun flushNow(): Result<Unit> {
        debounceJob?.cancel()
        debounceJob = null
        return runOneFlush()
    }

    suspend fun closeAndFlush(): Result<Unit> = flushNow()

    fun closeAndFlushAsync() {
        scope.launch { runOneFlush() }
    }

    private suspend fun runOneFlush(): Result<Unit> = flushMutex.withLock {
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
                    GlobalPopupManager.showError("Save failed: ${e.message}. Reloading page.")
                    onFailed()
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