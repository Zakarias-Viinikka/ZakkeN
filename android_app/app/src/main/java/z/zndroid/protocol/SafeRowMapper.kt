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
     * Maps a Row to a List of ColumnValues by verifying names against the table definition.
     * 
     * @param row The Row returned by a Rust 'new_xxx_row' helper.
     * @param columnDefs The full list of ColumnDefs (including 'id') from Rust 'xxx_columns'.
     * @param expectedNames The sequence of column names we expect to be in the Row (in order).
     * @return A list of ColumnValues ready for DbManager.insertData.
     */
    fun mapRow(
        row: Row,
        columnDefs: List<ColumnDef>,
        expectedNames: List<String>,
        skipId: Boolean = true
    ): List<ColumnValue> {
        // Rust row builders usually skip the auto-increment 'id' column (usually at index 0).
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
                    "Column mismatch at index $defIndex! Rust library returned '$actualName' but Kotlin expected '$expectedName'. " +
                    "Positional mapping is unsafe."
                )
            }

            ColumnValue(actualName, col)
        }
    }
}
