package z.zndroid.markdown_conversion.main_for_on_link_click

import android.util.Log
import z.zndroid.components.GlobalPopupManager

//! Dummy command [[Install x|...]]. Only logs and shows an info popup.

private const val TAG = "LinkCommandInstallX"

fun runInstallX(rest: String): Result<Unit> {
    Log.i(TAG, "Install x ran, rest=$rest")
    GlobalPopupManager.show("Install x (placeholder)")
    return Result.success(Unit)
}
