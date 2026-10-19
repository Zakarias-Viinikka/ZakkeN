package z.zndroid

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import z.zndroid.Tests.SchemaSyncTest

/**
 * Instrumented test that will fail during 'connectedCheck' if schemas drift.
 */
@RunWith(AndroidJUnit4::class)
class SchemaSyncVerification {
    @Test
    fun verifySchemaSync() = runBlocking {
        // Initialize DB if needed (or at least wait for it)
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        DbManager.init(context)
        DbManager.awaitReady()

        val test = SchemaSyncTest()
        val result = test.run()

        assertTrue(
            "Schema Sync Failed!\n${result.message}\n${result.errorContext}",
            result.isSuccess
        )
    }
}
