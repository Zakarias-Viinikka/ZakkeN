package z.zndroid.Tests

/**
 * A central registry for all internal tests.
 * To add a new test, simply create a new [AppTest] implementation and add it to this list.
 */
object RegisterTests {
    val allTests: List<AppTest> = listOf(
        SessionMonotonicityTest(),
        SchemaSyncTest(),
        PageDataSyncTest(),
        TransactionTest()
    )
}

/**
 * Example Test: Verifies that the session ID increments correctly.
 */
class SessionMonotonicityTest : AppTest {
    override val name = "Session Monotonicity"

    override suspend fun run(): TestResult {
        // If DB isn't ready (build time), skip this test
        if (z.zndroid.DbManager.executeNative { }.isFailure) {
            return TestResult(name, true, "Skipped (DB not ready)")
        }

        return try {
            val s1 = z.zndroid.Storage.SessionManager.currentSessionId.toLong()
            z.zndroid.Storage.SessionManager.incrementAndStore()
            // Give it a tiny bit of time as incrementAndStore is an async launch
            kotlinx.coroutines.delay(50) 
            val s2 = z.zndroid.Storage.SessionManager.currentSessionId.toLong()
            
            if (s2 > s1) {
                TestResult(name, true, "Counter successfully moved from $s1 to $s2")
            } else {
                TestResult(name, false, "Counter did not increment. Value remained at $s1")
            }
        } catch (e: Exception) {
            TestResult(name, false, "Exception occurred", e.stackTraceToString())
        }
    }
}
