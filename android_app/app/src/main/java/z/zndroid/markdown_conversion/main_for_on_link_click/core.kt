package z.zndroid.markdown_conversion.main_for_on_link_click

import android.util.Log
import z.zndroid.components.GlobalPopupManager

/*
 * Entry for a click on a [[command|rest]] link.
 *
 * Caller pov: call [mainOnLinkClick]. It finds the handler for that command,
 * runs it with `rest`, and shows a popup if it fails. Nothing is swallowed.
 *
 * To add a command: make a new file in this folder and add one line to the
 * `when` below.
 *
 * Android-only shape: `page_nav` is the only real command and it needs the
 * navigation callback, so `onOpenPage` is a required argument. Web's
 * `plexset_navigation_request` has no Android equivalent.
 */

private const val TAG = "LinkCommand"

fun mainOnLinkClick(command: String, rest: String, onOpenPage: (String) -> Unit) {
    val result: Result<Unit> = when (command) {
        "Install x" -> runInstallX(rest)
        "Install y" -> runInstallY(rest)
        "Install c" -> runInstallC(rest)
        "Install all things" -> runInstallAllThings(rest)
        "PageNav" -> runPageNav(rest, onOpenPage)
        else -> Result.failure(
            OnLinkClickError(
                errMsg = "unknown link command \"$command\" (rest was \"$rest\")",
                method = "run_matching_command",
                filepath = "main_for_on_link_click/core.kt",
            )
        )
    }
    result.onFailure { e ->
        val err = (e as? OnLinkClickError) ?: OnLinkClickError(
            errMsg = e.message ?: e.toString(),
            method = "main_on_link_click",
            filepath = "main_for_on_link_click/core.kt",
        )
        Log.e(
            TAG,
            "link click failed: err_msg=${err.errMsg} method=${err.method} filepath=${err.filepath}",
        )
        GlobalPopupManager.showError("Link command failed: ${err.errMsg}")
    }
}
