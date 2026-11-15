package z.zndroid.Tests

/**
 * The single list of [AppTest] implementations that [BuildVerificationTest]
 * runs on the host JVM via `./gradlew test`.
 *
 * This is separate from [DevCheckRegistry], which is the list the Dev Tools
 * page runs on a device.
 *
 * Schema drift detection is handled by schema_drift_detector/, not by a
 * test in this list. SchemaSyncTest checks the on-device DB against the
 * LATEST snapshot, which is only meaningful on the device.
 */
object RegisterTests {
    val allTests: List<AppTest> = listOf(
        MigrationTest(),
        TransactionTest(),
        SessionMonotonicityTest(),
    )
}