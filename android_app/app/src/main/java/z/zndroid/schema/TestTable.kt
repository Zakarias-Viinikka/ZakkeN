package z.zndroid.schema

import uniffi.protocol.ColumnDef
import uniffi.protocol.ColumnType
import uniffi.protocol.idColumn
import uniffi.protocol.defaultCol

fun testTableDef(): List<ColumnDef> {
    val x = listOf(
        idColumn(),
        defaultCol(ColumnType.Text, "name"),
        defaultCol(ColumnType.Integer, "age")
    )
    return x
}

fun testTableName(): String {
    return "test_table"
}
