package z.zndroid.sync.owner_of_interacting_with_server

import android.util.Log
import kotlinx.coroutines.launch
import rustlib.server_link.LoginInfo
import rustlib.server_link.LoginOutcome
import rustlib.server_link.RegisterOutcome
import rustlib.server_link.ServerLink
import rustlib.server_link.setDriftVersion
import uniffi.protocol.Col
import uniffi.protocol.DeleteRowIn
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.actions.kickSend
import z.zndroid.Storage.StorageKey
import z.zndroid.components.GlobalPopupManager
import z.zndroid.db.schema.SeenServerLettersTableSchema

private const val TAG = "SyncLogin"
private const val SERVER_NAME_KEY = "server_user_name"
internal const val CLIENT_DRIFT_KEY = "client_drift_version"
private const val DRIFT_POPUP_KEY = "should_i_drift_popup"

/**
 * Log in after the socket (re)opens. Called by the receive loop, so letters
 * pushed during login wait in the inbox until state is Connected (same
 * ordering as web). Returns true if login succeeded.
 */
internal suspend fun loginAfterReconnect(link: ServerLink): Boolean {
    return try {
        login(link)
        true
    } catch (e: Exception) {
        Log.e(TAG, "login failed", e)
        GlobalPopupManager.showError("login failed: ${e.message ?: e}")
        setConnState(ConnState.FailedToConnect)
        false
    }
}

/** The server said our dataset is no longer current. */
internal fun spawnOwnerOfDriftChanged() {
    syncScope.launch {
        for (changed in SpontaneousInbox.driftChanged) {
            enterLonewolf(changed.driftReason.ifEmpty { null })
        }
    }
}

private suspend fun login(link: ServerLink) {
    val setupName = StorageAccess.getUserId()
    if (setupName.isEmpty()) throw IllegalStateException("USER_ID not set or empty")

    val saved = readKeyValue(SERVER_NAME_KEY)
    if (saved != null) {
        when (val outcome = link.login(saved)) {
            is LoginOutcome.LoggedIn -> {
                afterLogin(outcome.info)
                return
            }
            is LoginOutcome.NoSuchUser -> {
                Log.w(TAG, "saved server name '$saved' doesn't exist on server; clearing and re-registering")
                writeKeyValue(SERVER_NAME_KEY, "")
                clearSeenServerLetters()
                SeenLetters.clearMemory()
            }
            is LoginOutcome.Other -> throw IllegalStateException(outcome.errMsg)
        }
    }

    when (val registered = link.register(setupName)) {
        is RegisterOutcome.Registered -> {
            writeKeyValue(SERVER_NAME_KEY, setupName)
            when (val outcome = link.login(setupName)) {
                is LoginOutcome.LoggedIn -> afterLogin(outcome.info)
                is LoginOutcome.NoSuchUser -> throw IllegalStateException("login after register: no such user")
                is LoginOutcome.Other -> throw IllegalStateException("login after register: ${outcome.errMsg}")
            }
        }
        is RegisterOutcome.AlreadyRegistered -> {
            GlobalPopupManager.showError(
                "Server says '$setupName' is already taken. Pick a different user name and restart the app."
            )
            StorageAccess.setValue(StorageKey.USER_ID, "")
            throw IllegalStateException("server name '$setupName' is already taken")
        }
        is RegisterOutcome.Other -> throw IllegalStateException(registered.errMsg)
    }
}

internal suspend fun afterLogin(info: LoginInfo) {
    val stored = readKeyValue(CLIENT_DRIFT_KEY)
    if (stored == null || stored == info.driftVersion) {
        if (stored == null) writeKeyValue(CLIENT_DRIFT_KEY, info.driftVersion)
        setDriftVersion(info.driftVersion)
        writeKeyValue(DRIFT_POPUP_KEY, "true")
        if (connState.value == ConnState.LoggingIn) {
            setConnState(ConnState.Connected)
            kickSend()
        }
    } else {
        enterLonewolf(info.driftReason.ifEmpty { null })
    }
}

/**
 * Sync is stopped because our drift version and the server's don't
 * match. Flips the state at once, then shows the reason once.
 */
internal fun enterLonewolf(reason: String?) {
    val next = when (connState.value) {
        ConnState.Connected, ConnState.Lonewolf, ConnState.LoggingIn -> ConnState.Lonewolf
        else -> ConnState.LonewolfAndOffline
    }
    setConnState(next)
    syncScope.launch {
        val should = try {
            readKeyValue(DRIFT_POPUP_KEY) != "false"
        } catch (e: Exception) {
            Log.w(TAG, "enterLonewolf: reading $DRIFT_POPUP_KEY failed", e)
            true
        }
        if (!should) return@launch
        GlobalPopupManager.showError(
            if (reason == null) "out of sync with server" else "out of sync with server: $reason"
        )
        try {
            writeKeyValue(DRIFT_POPUP_KEY, "false")
        } catch (e: Exception) {
            Log.w(TAG, "enterLonewolf: writing $DRIFT_POPUP_KEY failed", e)
        }
    }
}

internal suspend fun clearSeenServerLetters() {
    DbManager.withTransaction {
        val rows = DbManager.getData(
            GetDataIn(
                SeenServerLettersTableSchema.TABLE_NAME,
                SelectArguments.Single(SelectArgument.All),
                listOf(SeenServerLettersTableSchema.id.name)
            )
        ).getOrThrow().rows
        for (row in rows) {
            val id = (row.cols.firstOrNull() as? Col.Integer)?.v1?.toString() ?: continue
            DbManager.deleteRow(DeleteRowIn(SeenServerLettersTableSchema.TABLE_NAME, id)).getOrThrow()
        }
    }.getOrThrow()
}
