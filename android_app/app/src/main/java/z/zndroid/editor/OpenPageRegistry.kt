package z.zndroid.editor

import java.util.concurrent.ConcurrentHashMap

/**
 * The page ViewPage has open right now, if any. The receive path merges
 * remote letters into its live doc (web's pick_boss_for_page) so the
 * session and the db never diverge.
 */
object OpenPageRegistry {
    private val sessions = ConcurrentHashMap<String, PageSession>()

    fun register(session: PageSession) {
        sessions[session.pageId] = session
    }

    /** Only removes `session` itself, never a newer session for the same page. */
    fun unregister(session: PageSession) {
        sessions.remove(session.pageId, session)
    }

    fun get(pageId: String): PageSession? = sessions[pageId]
}
