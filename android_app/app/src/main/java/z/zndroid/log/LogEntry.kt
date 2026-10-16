package z.zndroid.log

data class LogEntry(
    val timestamp: Long = System.currentTimeMillis(),
    val level: String,
    val category: String,
    val source: String,
    val sessionId: String,
    val message: String,
    val details: ByteArray? = null,
    val detailsType: String? = null
)
