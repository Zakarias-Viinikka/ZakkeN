package z.zndroid.log

import java.time.LocalDateTime
import java.time.ZoneOffset

/**
 * Helpers for date/time conversion within the logging and query context.
 */
object LogDateHelpers {

    /**
     * Converts two LocalDateTime objects into a Pair of Unix timestamps (seconds).
     * Useful for range-based SQLite queries.
     */
    fun toUnixRange(start: LocalDateTime, end: LocalDateTime): Pair<Long, Long> {
        val startUnix = start.toEpochSecond(ZoneOffset.UTC)
        val endUnix = end.toEpochSecond(ZoneOffset.UTC)
        return Pair(startUnix, endUnix)
    }

    /**
     * Converts a single LocalDateTime to a Unix timestamp.
     */
    fun toUnix(dateTime: LocalDateTime): Long {
        return dateTime.toEpochSecond(ZoneOffset.UTC)
    }
}
