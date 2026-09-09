package z.zndroid.MainPages.ViewPage

/**
 * Determines if the current input sequence should trigger the creation of a new block.
 * Logic: "is the character above me just a linebreak" AND "user pressed linebreak".
 */
fun isItTimeToMakeANewBlock(oldText: String, newText: String): Boolean {
    // Check if the old text ended with a newline and the new text adds another one
    return oldText.endsWith("\n") && newText.endsWith("\n\n")
}
