package z.zndroid.Tests

/**
 * Result of an internal application test.
 */
data class TestResult(
    val name: String,
    val isSuccess: Boolean,
    val message: String = "",
    val errorContext: String? = null
)

/**
 * Interface that all internal tests must implement to be automatically
 * discovered and run by the [InternalAppTests] system.
 */
interface AppTest {
    val name: String
    suspend fun run(): TestResult
}
