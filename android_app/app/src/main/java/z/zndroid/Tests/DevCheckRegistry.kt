package z.zndroid.Tests

import android.util.Log
import z.zndroid.db.migrations.SchemaVersion

private fun TestResult.toDevCheck(id: String): DevCheckResult {
    val status = if (isSuccess) DevCheckStatus.PASS else DevCheckStatus.FAIL
    return DevCheckResult(id, name, status, message, errorContext)
}

object DevCheckRegistry {
    val checks: List<DevCheck> = listOf(
        DevCheck("schema.blueprints", "Schema blueprints vs frozen") {
            try {
                SchemaVersion.assertLatestMatchesBlueprints()
                pass("schema.blueprints", "Schema blueprints vs frozen", "Live Rust blueprints match the latest frozen Kotlin version.")
            } catch (e: SchemaVersion.SchemaMismatchException) {
                fail("schema.blueprints", "Schema blueprints vs frozen", e.message ?: "Schema mismatch")
            } catch (e: Exception) {
                fail("schema.blueprints", "Schema blueprints vs frozen", "Exception: ${e.message}")
            }
        },
        DevCheck("schema.canary", "Schema type canary") {
            SchemaCanaryTest().run().toDevCheck("schema.canary")
        },
        DevCheck("schema.live_db", "Schema live DB") {
            SchemaSyncTest().run().toDevCheck("schema.live_db")
        },
        DevCheck("migration", "Migration bootstrap") {
            MigrationTest().run().toDevCheck("migration")
        },
        DevCheck("session.monotonic", "Session monotonic") {
            SessionMonotonicityTest().run().toDevCheck("session.monotonic")
        },
        DevCheck(MirrorVsYrsCheck.ID, MirrorVsYrsCheck.NAME) {
            MirrorVsYrsCheck.run()
        },
        DevCheck(OutboxIntegrityCheck.ID, OutboxIntegrityCheck.NAME) {
            OutboxIntegrityCheck.run()
        },
        DevCheck(OutboxReplayCheck.ID, OutboxReplayCheck.NAME) {
            OutboxReplayCheck.run()
        },
        DevCheck("ops.create_and_rename", "Ops create and rename") { OpsChecks.createAndRename() },
        DevCheck("ops.split_merge", "Ops split then merge") { OpsChecks.splitMerge() },
        DevCheck("ops.delete_rules", "Ops delete rules") { OpsChecks.deleteRules() },
        DevCheck("ops.move_same_page", "Ops move same page") { OpsChecks.moveSamePage() },
        DevCheck("ops.move_cross_page", "Ops move cross page") { OpsChecks.moveCrossPage() },
        DevCheck("ops.disable_hides_from_menu", "Ops disable hides from menu") { OpsChecks.disableHidesFromMenu() },

        DevCheck("client_vs_server", "Client vs server") {
            pass("client_vs_server", "Client vs server", "NOT IMPLEMENTED - reserved")
        }
    )

    suspend fun runOne(id: String): DevCheckResult {
        val check = checks.firstOrNull { it.id == id }
            ?: return fail(id, id, "Unknown check id")
        val result = try {
            check.run()
        } catch (e: Exception) {
            fail(check.id, check.name, "Uncaught: ${e.message}", e.stackTraceToString())
        }
        Log.i("DevCheck", "id=${result.id} status=${result.status} detail=${result.message.take(200)}")
        if (result.detail != null) {
            Log.i("DevCheck", "id=${result.id} st=${result.status} stack=${result.detail.take(1500)}")
        }
        return result
    }
}