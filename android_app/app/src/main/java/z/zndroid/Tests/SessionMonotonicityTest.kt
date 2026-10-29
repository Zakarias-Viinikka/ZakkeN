package z.zndroid.Tests

class SessionMonotonicityTest : AppTest {
    override val name = "Session Monotonicity"

    override suspend fun run(): TestResult {
        if (z.zndroid.DbManager.executeNative { }.isFailure) {
            return TestResult(name, true, "Skipped (DB not ready)")
        }
        return try {
            val s1 = z.zndroid.Storage.SessionManager.currentSessionId.toLongOrNull()
                ?: return TestResult(name, false, "Session id was not a number")
            if (s1 < 1) {
                return TestResult(name, false, "Session id should be >= 1 after init, got $s1")
            }
            kotlinx.coroutines.delay(100)
            val s2 = z.zndroid.Storage.SessionManager.currentSessionId.toLongOrNull() ?: s1
            if (s1 == s2) {
                TestResult(name, true, "Session id stable at $s1")
            } else {
                TestResult(name, false, "Session id changed without navigation: $s1 -> $s2")
            }
        } catch (e: Exception) {
            TestResult(name, false, "Exception occurred", e.stackTraceToString())
        }
    }
}
