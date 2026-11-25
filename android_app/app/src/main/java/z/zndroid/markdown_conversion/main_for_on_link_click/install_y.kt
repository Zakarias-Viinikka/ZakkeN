package z.zndroid.markdown_conversion.main_for_on_link_click

import android.util.Log
import z.zndroid.components.GlobalPopupManager

//! Dummy command [[Install y|...]]. Only logs and shows an info popup.

private const val TAG = "LinkCommandInstallY"

fun runInstallY(rest: String): Result<Unit> {
    Log.i(TAG, "Install y ran, rest=$rest")
    GlobalPopupManager.show("Install y (placeholder)")
    return Result.success(Unit)
}
