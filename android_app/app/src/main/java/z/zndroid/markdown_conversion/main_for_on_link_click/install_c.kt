package z.zndroid.markdown_conversion.main_for_on_link_click

import android.util.Log
import z.zndroid.components.GlobalPopupManager

//! Dummy command [[Install c|...]]. Only logs and shows an info popup.

private const val TAG = "LinkCommandInstallC"

fun runInstallC(rest: String): Result<Unit> {
    Log.i(TAG, "Install c ran, rest=$rest")
    GlobalPopupManager.show("Install c (placeholder)")
    return Result.success(Unit)
}
