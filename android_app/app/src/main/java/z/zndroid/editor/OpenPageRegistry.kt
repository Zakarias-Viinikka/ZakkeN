package z.zndroid.editor

import kotlinx.coroutines.Job
import java.util.concurrent.ConcurrentHashMap

/**
 * The page ViewPage has open right now, if any. The receive path merges
 * remote letters into its live doc (web's pick_boss_for_page) so the
 * session and the db never diverge.
 */
object OpenPageRegistry {
    private val sessions = ConcurrentHashMap<String, PageSession>()
    private val closingJobs = ConcurrentHashMap<String, Job>()

    fun register(session: PageSession) {
        sessions[session.pageId] = session
    }

    /** Only removes `session` itself, never a newer session for the same page. */
    fun unregister(session: PageSession) {
        sessions.remove(session.pageId, session)
    }

    /**
     * Unregisters [session] and asynchronously flushes pending edits, destroys
     * its BossOfYrs, and destroys its PageBatcher. Tracks the job per pageId so
     * a quickly reopened ViewPage can await it before reading the DB.
     */
    fun unregisterAndClose(session: PageSession) {
        unregister(session)
        val job = session.destroyAfterFlushAsync()
        closingJobs[session.pageId] = job
        job.invokeOnCompletion {
            closingJobs.remove(session.pageId, job)
        }
    }

    suspend fun awaitPendingClose(pageId: String) {
        while (true) {
            val job = closingJobs[pageId] ?: return
            job.join()
        }
    }

    fun get(pageId: String): PageSession? = sessions[pageId]
}