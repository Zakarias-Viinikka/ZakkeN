package z.zndroid.markdown_conversion.main_for_on_link_click

import android.util.Log

//! Dummy command [[Install all things|...]]. Runs the other install commands.

private const val TAG = "LinkCommandInstallAll"

fun runInstallAllThings(rest: String): Result<Unit> {
    Log.i(TAG, "Install all things started, rest=$rest")
    runInstallX(rest).getOrElse { return Result.failure(it) }
    runInstallY(rest).getOrElse { return Result.failure(it) }
    runInstallC(rest).getOrElse { return Result.failure(it) }
    return Result.success(Unit)
}
