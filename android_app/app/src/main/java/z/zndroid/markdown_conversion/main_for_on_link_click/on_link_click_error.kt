package z.zndroid.markdown_conversion.main_for_on_link_click

/**
 * Error returned by a link command. Same three fields as the Rust original:
 * what went wrong, which method, which file.
 *
 * Android-only difference: extends [Exception] so it can go through
 * `Result.failure`. `message` is `errMsg`, so callers that only look at
 * `Throwable.message` see the same string.
 */
class OnLinkClickError(
    val errMsg: String,
    val method: String,
    val filepath: String,
) : Exception(errMsg)