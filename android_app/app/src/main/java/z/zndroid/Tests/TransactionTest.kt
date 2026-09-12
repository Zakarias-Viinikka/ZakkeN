package z.zndroid.Tests

import uniffi.protocol.*
import z.zndroid.DbManager
import rustlib.client_table_blueprints.keyValueStorageColumns
import rustlib.client_table_blueprints.newKeyValueItem
import z.zndroid.protocol.SafeRowMapper
import java.util.UUID

/**
 * Verifies that transactions in DbManager work as expected:
 * 1. Atomicity: Rollback on exception.
 * 2. Re-entrancy: Nested withTransaction calls.
 * 3. Persistence: Data is saved on success.
 */
class TransactionTest : AppTest {
    override val name = "Database Transactions"

    override suspend fun run(): TestResult {
        if (DbManager.executeNative { }.isFailure) {
            return TestResult(name, true, "Skipped (DB not ready)")
        }

        val testKey = "transaction_test_${UUID.randomUUID()}"
        
        return try {
            // 1. Test Rollback
            val rollbackResult = DbManager.withTransaction {
                insertKeyValue(testKey, "should_be_rolled_back")
                throw IntentionalRollbackException()
            }
            
            if (rollbackResult.isSuccess) {
                return TestResult(name, false, "Transaction reported success despite exception")
            }
            
            val rummageRes = z.zndroid.Storage.StorageAccess.rummage_in_storage(z.zndroid.Storage.StorageKey.CUSTOM(testKey))
            if (rummageRes !is z.zndroid.Storage.RummageResult.NotFound) {
                return TestResult(name, false, "Rollback failed: key '$testKey' was persisted")
            }

            // 2. Test Success & Re-entrancy
            val successResult = DbManager.withTransaction {
                insertKeyValue(testKey, "stage_1")
                
                // Nested transaction (re-entrant)
                DbManager.withTransaction {
                    insertKeyValue(testKey + "_nested", "nested_val")
                }.getOrThrow()
                
                "success_marker"
            }

            if (successResult.isFailure) {
                return TestResult(name, false, "Successful transaction reported failure: ${successResult.exceptionOrNull()}")
            }

            val val1 = z.zndroid.Storage.StorageAccess.rummage_in_storage(z.zndroid.Storage.StorageKey.CUSTOM(testKey))
            val val2 = z.zndroid.Storage.StorageAccess.rummage_in_storage(z.zndroid.Storage.StorageKey.CUSTOM(testKey + "_nested"))

            if (val1 is z.zndroid.Storage.RummageResult.StringValue && val1.value == "stage_1" &&
                val2 is z.zndroid.Storage.RummageResult.StringValue && val2.value == "nested_val") {
                TestResult(name, true, "Transactions passed: Rollback works, Re-entrancy works, Persistence works.")
            } else {
                TestResult(name, false, "Persistence verification failed. Val1: $val1, Val2: $val2")
            }

        } catch (e: Exception) {
            TestResult(name, false, "Unexpected error in test harness", e.stackTraceToString())
        }
    }

    private suspend fun insertKeyValue(key: String, value: String) {
        val row = newKeyValueItem(key, value)
        val values = SafeRowMapper.mapRow(
            row = row,
            columnDefs = keyValueStorageColumns(),
            expectedNames = listOf("key", "value"),
            skipId = false
        )
        DbManager.insertData(InsertDataIn("key_value_storage", values)).getOrThrow()
    }

    private class IntentionalRollbackException : Exception("Intentional rollback for testing")
}
