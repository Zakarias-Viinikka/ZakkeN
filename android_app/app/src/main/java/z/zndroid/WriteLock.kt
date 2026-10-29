package z.zndroid

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * One writer at a time across the whole app.
 *
 * Acquired by every public action (create page, rename, disable) and by the
 * batcher flush. Lock order is always WriteLock then DbManager's mutex.
 * Nothing inside a locked section may try to take WriteLock again.
 */
object WriteLock {
    private val mutex = Mutex()
    suspend fun <T> withLock(block: suspend () -> T): T = mutex.withLock { block() }
}
