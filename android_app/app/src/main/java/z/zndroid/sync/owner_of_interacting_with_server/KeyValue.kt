package z.zndroid.sync.owner_of_interacting_with_server

import rustlib.client_table_blueprints.newKeyValueItem
import rustlib.client_table_blueprints.newTableKeyValueStorage
import uniffi.protocol.Col
import uniffi.protocol.EditColInRowWhereIn
import uniffi.protocol.GetDataIn
import uniffi.protocol.InsertDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.db.schema.KeyValueStorageTableSchema
import z.zndroid.protocol.SafeRowMapper

/**
 * Value stored under `key` in key_value_storage. Null when the key is
 * missing or its value is empty (web treats empty as unset too).
 * Throws if the db read fails.
 */
internal suspend fun readKeyValue(key: String): String? {
    val columns = listOf(KeyValueStorageTableSchema.value.name)
    val out = DbManager.getData(
        GetDataIn(
            KeyValueStorageTableSchema.TABLE_NAME,
            SelectArguments.Single(SelectArgument.XEqualY(KeyValueStorageTableSchema.key.name, key)),
            columns
        )
    ).getOrThrow()
    val col = out.rows.firstOrNull()?.cols?.firstOrNull() ?: return null
    val text = (col as? Col.Text)?.v1 ?: return null
    return text.ifEmpty { null }
}

/**
 * Upsert `value` under `key` in key_value_storage. Throws if the write fails.
 */
internal suspend fun writeKeyValue(key: String, value: String) {
    DbManager.withTransaction {
        val existing = DbManager.getData(
            GetDataIn(
                KeyValueStorageTableSchema.TABLE_NAME,
                SelectArguments.Single(SelectArgument.XEqualY(KeyValueStorageTableSchema.key.name, key)),
                emptyList()
            )
        ).getOrThrow()

        if (existing.rows.isNotEmpty()) {
            DbManager.editColInRowWhere(
                EditColInRowWhereIn(
                    tableName = KeyValueStorageTableSchema.TABLE_NAME,
                    whereClause = SelectArguments.Single(
                        SelectArgument.XEqualY(KeyValueStorageTableSchema.key.name, key)
                    ),
                    column = KeyValueStorageTableSchema.value.name,
                    newValue = Col.Text(value)
                )
            ).getOrThrow()
        } else {
            val row = newKeyValueItem(key, value)
            val values = SafeRowMapper.mapRow(
                row = row,
                columnDefs = newTableKeyValueStorage(),
                expectedNames = listOf("key", "value"),
                skipId = true
            )
            DbManager.insertData(InsertDataIn(KeyValueStorageTableSchema.TABLE_NAME, values)).getOrThrow()
        }
    }.getOrThrow()
}