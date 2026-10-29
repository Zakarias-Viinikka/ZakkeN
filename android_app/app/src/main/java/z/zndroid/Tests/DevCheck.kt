package z.zndroid.Tests

enum class DevCheckStatus { NOT_RUN, RUNNING, PASS, FAIL }

data class DevCheckResult(
    val id: String,
    val name: String,
    val status: DevCheckStatus = DevCheckStatus.NOT_RUN,
    val message: String = "",
    val detail: String? = null
)

data class DevCheck(
    val id: String,
    val name: String,
    val run: suspend () -> DevCheckResult
)

fun pass(id: String, name: String, message: String = ""): DevCheckResult =
    DevCheckResult(id, name, DevCheckStatus.PASS, message)

fun fail(id: String, name: String, message: String, detail: String? = null): DevCheckResult =
    DevCheckResult(id, name, DevCheckStatus.FAIL, message, detail)
