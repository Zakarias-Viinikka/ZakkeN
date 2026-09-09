package z.zndroid.Tests

import androidx.compose.runtime.mutableStateMapOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/**
 * Global runner for internal tests. Stores results in a state observable by Compose.
 */
object InternalAppTests {
    /**
     * Map of Test Name -> TestResult. Observed by the TestResultsPage.
     */
    val results = mutableStateMapOf<String, TestResult>()
    
    private val scope = CoroutineScope(Dispatchers.Default)

    /**
     * Runs all registered tests and updates the [results] map.
     */
    fun runAll() {
        RegisterTests.allTests.forEach { test ->
            scope.launch {
                // Initialize as "running" or reset
                val result = test.run()
                results[test.name] = result
            }
        }
    }
    
    /**
     * Clears all cached test results.
     */
    fun clear() {
        results.clear()
    }
}
