package com.example.feellabs.notes

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.snapshots.SnapshotStateList
import java.util.concurrent.atomic.AtomicLong

class BlockModel(
    val id: Long,
    val text: TextFieldState,
    var height: Float,
)

class PageModel(
    val id: Long,
    val title: TextFieldState,
    val blocks: SnapshotStateList<BlockModel>,
) {
    private val blockCounter = AtomicLong(1_000_000L)
    fun newBlockId(): Long = blockCounter.getAndIncrement()
}

class NoteStore {
    private val nextId = AtomicLong(1)
    val pages: SnapshotStateList<PageModel> = mutableStateListOf()

    init {
        seed()
    }

    fun newId(): Long = nextId.getAndIncrement()

    fun pageById(id: Long): PageModel? = pages.firstOrNull { it.id == id }

    private fun seed() {
        pages += PageModel(
            id = newId(),
            title = TextFieldState("Field notes"),
            blocks = mutableStateListOf(
                BlockModel(newId(), TextFieldState("Booked the early train. The afternoon is open for the museum."), 60f),
                BlockModel(newId(), TextFieldState("Ask about the annual plan — it saves a bit over monthly."), 44f),
                BlockModel(newId(), TextFieldState("Two people for dinner on Friday, not three."), 44f),
                BlockModel(newId(), TextFieldState("The schema migration is the risky part; test on staging first."), 60f),
                BlockModel(newId(), TextFieldState("Short walk after lunch if the weather holds."), 44f),
                BlockModel(newId(), TextFieldState("Renew the library card before the end of the month."), 60f),
            ),
        )
        pages += PageModel(
            id = newId(),
            title = TextFieldState("Second page"),
            blocks = mutableStateListOf(
                BlockModel(newId(), TextFieldState("A second page, seeded from the web mock texts."), 44f),
                BlockModel(newId(), TextFieldState("Blocks can be inserted and deleted here too."), 44f),
            ),
        )
    }
}
