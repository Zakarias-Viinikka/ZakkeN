package z.zndroid.log

import java.time.LocalDateTime
import java.time.ZoneId

/**
 * Helpers for date/time conversion within the logging and query context.
 *
 * Both functions return Unix epoch MILLISECONDS, matching
 * [LogEntry.timestamp] (`System.currentTimeMillis()`).
 *
 * The input [LocalDateTime] is interpreted in the device's current zone,
 * so a range picked from a local-time picker maps to the wall-clock window
 * the user chose.
 */
object LogDateHelpers {

    fun toUnixRange(start: LocalDateTime, end: LocalDateTime): Pair<Long, Long> {
        val zone = ZoneId.systemDefault()
        val startMs = start.atZone(zone).toInstant().toEpochMilli()
        val endMs = end.atZone(zone).toInstant().toEpochMilli()
        return Pair(startMs, endMs)
    }

    fun toUnix(dateTime: LocalDateTime): Long {
        return dateTime.atZone(ZoneId.systemDefault()).toInstant().toEpochMilli()
    }
}