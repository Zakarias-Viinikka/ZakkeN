package z.zndroid.Tests

import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.runBlocking

/**
 * A JUnit wrapper that runs your internal app tests during the Gradle build.
 * Note: If your tests require the native Rust libraries, they must be available
 * for the host architecture (JNA path) for this to pass at compile time.
 */
class BuildVerificationTest {

    @Test
    fun runInternalSuite() = runBlocking {
        println("--- Running Build-Time Verification ---")
        
        var allPassed = true
        val failures = mutableListOf<String>()

        RegisterTests.allTests.forEach { test ->
            print("Running ${test.name}... ")
            try {
                val result = test.run()
                if (result.isSuccess) {
                    println("PASSED")
                } else {
                    println("FAILED")
                    allPassed = false
                    failures.add("${test.name}: ${result.message}")
                }
            } catch (e: UnsatisfiedLinkError) {
                println("SKIPPED (Native library not found on host)")
            } catch (e: Exception) {
                println("ERROR")
                allPassed = false
                failures.add("${test.name}: Exception ${e.message}")
            }
        }

        if (!allPassed) {
            fail("Internal verification failed:\n" + failures.joinToString("\n"))
        }
        
        println("--- Verification Complete: ALL PASSED ---")
    }
}
