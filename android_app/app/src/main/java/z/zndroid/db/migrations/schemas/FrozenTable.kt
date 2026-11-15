package z.zndroid.db.migrations.schemas

import uniffi.protocol.ColumnDef
import uniffi.protocol.ForeignKeyDef

/**
 * One table frozen in a schema snapshot. `name`, ordered columns, and
 * ordered foreign keys. This is the shape `schema_drift_detector/` emits
 * from live Rust when you run `check.sh emit-snapshot <N>`.
 */
data class FrozenTable(
    val name: String,
    val columns: List<ColumnDef>,
    val foreignKeys: List<ForeignKeyDef> = emptyList(),
)
