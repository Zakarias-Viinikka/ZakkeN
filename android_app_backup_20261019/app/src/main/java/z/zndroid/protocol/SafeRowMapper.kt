package z.zndroid.protocol

import uniffi.protocol.ColumnDef
import uniffi.protocol.ColumnValue
import uniffi.protocol.Row

/**
 * Ensures that mapping a Rust-generated Row to SQLite ColumnValues is safe
 * and doesn't rely on fragile positional assumptions.
 */
object SafeRowMapper {

    /**
     * Maps/validates a List<ColumnValue> returned by Rust 'new_xxx_row' helpers against table definition.
     */
    fun mapRow(
        row: List<ColumnValue>,
        columnDefs: List<ColumnDef>,
        expectedNames: List<String>,
        skipId: Boolean = true
    ): List<ColumnValue> {
        val offset = if (skipId) 1 else 0
        
        if (row.size != expectedNames.size) {
            throw IllegalStateException(
                "Row column count (${row.size}) does not match expected count (${expectedNames.size})"
            )
        }

        row.forEachIndexed { index, colValue ->
            val defIndex = index + offset
            if (defIndex >= columnDefs.size) {
                throw IllegalStateException("Column index $defIndex out of bounds for table definition")
            }

            val actualName = columnDefs[defIndex].name
            val expectedName = expectedNames[index]

            if (actualName != expectedName) {
                throw IllegalStateException(
                    "Column mismatch at index $defIndex! Rust library returned '$actualName' but Kotlin expected '$expectedName'."
                )
            }
        }

        return row
    }

    /**
     * Maps a Row to a List of ColumnValues by verifying names against the table definition.
     */
    fun mapRow(
        row: Row,
        columnDefs: List<ColumnDef>,
        expectedNames: List<String>,
        skipId: Boolean = true
    ): List<ColumnValue> {
        val offset = if (skipId) 1 else 0
        
        if (row.cols.size != expectedNames.size) {
            throw IllegalStateException(
                "Row column count (${row.cols.size}) does not match expected count (${expectedNames.size})"
            )
        }

        return row.cols.mapIndexed { index, col ->
            val defIndex = index + offset
            if (defIndex >= columnDefs.size) {
                throw IllegalStateException("Column index $defIndex out of bounds for table definition")
            }

            val actualName = columnDefs[defIndex].name
            val expectedName = expectedNames[index]

            if (actualName != expectedName) {
                throw IllegalStateException(
                    "Column mismatch at index $defIndex! Rust library returned '$actualName' but Kotlin expected '$expectedName'."
                )
            }

            ColumnValue(actualName, col)
        }
    }
}
