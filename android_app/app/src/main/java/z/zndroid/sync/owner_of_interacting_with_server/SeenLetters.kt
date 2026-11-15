package z.zndroid.sync.owner_of_interacting_with_server

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import rustlib.client_table_blueprints.newSeenServerLetterRow
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.InsertDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.db.schema.SeenServerLettersTableSchema

/**
 * Server letter ids this client has already applied. The in-memory set
 * is how duplicate pushes are dropped; seen_server_letters is what
 * survives a restart.
 */
internal object SeenLetters {
    private val ids = HashSet<Long>()
    private val lock = Mutex()

    /** Replace the in-memory set with what the table holds. Throws on db failure. */
    suspend fun load() {
        val out = DbManager.getData(
            GetDataIn(
                SeenServerLettersTableSchema.TABLE_NAME,
                SelectArguments.Single(SelectArgument.All),
                listOf(SeenServerLettersTableSchema.letterId.name)
            )
        ).getOrThrow()
        lock.withLock {
            ids.clear()
            for (row in out.rows) {
                val col = row.cols.firstOrNull() as? Col.Integer ?: continue
                ids.add(col.v1)
            }
        }
    }

    suspend fun contains(id: Long): Boolean = lock.withLock { ids.contains(id) }

    /** True if `id` was not there and now is. */
    suspend fun claim(id: Long): Boolean = lock.withLock { ids.add(id) }

    /** Undo a claim when the letter could not be applied. */
    suspend fun release(id: Long) {
        lock.withLock { ids.remove(id) }
    }

    /** Write `id` to seen_server_letters. Throws on db failure. */
    suspend fun persist(id: Long) {
        DbManager.insertData(
            InsertDataIn(SeenServerLettersTableSchema.TABLE_NAME, newSeenServerLetterRow(id))
        ).getOrThrow()
    }

    suspend fun clearMemory() {
        lock.withLock { ids.clear() }
    }
}
