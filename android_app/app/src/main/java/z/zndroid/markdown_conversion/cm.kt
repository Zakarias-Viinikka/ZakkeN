package z.zndroid.markdown_conversion

import android.util.Log
import android.webkit.JavascriptInterface
import android.webkit.ValueCallback
import android.webkit.WebView
import android.webkit.WebViewClient
import org.json.JSONArray
import org.json.JSONObject
import org.json.JSONTokener
import java.util.concurrent.ConcurrentHashMap

/**
 * The ONLY file that talks to the JS bridge (assets/markdown_web_output/cm_bridge.js
 * loaded by editor_host.html). Calls in: `evaluateJavascript` against
 * `window.cmBridge.*`. Events out: `editor_host.html` forwards the bridge's
 * CustomEvent "cm" into the Kotlin [CmHost] at `window.CmHost.postEvent(detailJson)`.
 * Everything outside this file sees typed Kotlin values, never JSON.
 *
 * Mirror of cqrs_state_synchronization/web_interface/src/markdown_conversion/cm.rs.
 * Kotlin has no `mod` mechanism, so the Rust `cm::*` namespacing is preserved
 * with an `object Cm`; every function name matches the Rust one.
 *
 * Android-only differences vs web (documented in INVENTED_ON_ANDROID.md):
 *  - The host is a WebView per block, not a Leptos <div>. `create` installs the
 *    JS bridge's [CmHost] and can be called before the page has loaded; later
 *    calls queue until the bridge is ready.
 *  - `getText` / `getSelection` are async (`evaluateJavascript` callbacks),
 *    not synchronous returns.
 *  - `getContentHeight` measures the rendered CSS px so the host Compose code
 *    can size the WebView.
 */

private const val TAG = "CmBridge"
const val CM_EDITOR_URL: String = "file:///android_asset/markdown_web_output/editor_host.html"

sealed class CmEvent {
    data class Changes(val changes: List<Change>, val head: Int) : CmEvent()
    data class Key(
        val key: String,
        val anchor: Int,
        val head: Int,
        val docLen: Int,
        val ranges: Int,
    ) : CmEvent()
    data class LinkClick(val command: String, val rest: String) : CmEvent()
    data class Selection(val head: Int) : CmEvent()
}

/** Title info for wikilink widgets. Key in the map = page id. */
data class PageTitle(val title: String, val disabled: Boolean)

object Cm {
    private val webViews = ConcurrentHashMap<String, WebView>()
    private val eventHandlers = ConcurrentHashMap<String, (CmEvent) -> Boolean>()
    private val readyIds = ConcurrentHashMap.newKeySet<String>()
    private val pendingOps = ConcurrentHashMap<String, MutableList<() -> Unit>>()

    /**
     * Create the editor inside [webView]. Must be called before the WebView loads
     * the host page. The caller is responsible for calling [destroy] when the
     * WebView goes away.
     *
     * [onEvent] returns true when a key press should be consumed (forwarded to
     * `preventDefault` on the JS CustomEvent). Non-key events always return false.
     * [onReady] runs after CM6 has been created and is safe for `setText` etc.
     */
    fun create(
        webView: WebView,
        id: String,
        initial: String,
        onEvent: (CmEvent) -> Boolean,
        titles: Map<String, PageTitle> = emptyMap(),
        onReady: (() -> Unit)? = null,
    ) {
        webViews[id] = webView
        eventHandlers[id] = onEvent
        webView.settings.javaScriptEnabled = true
        webView.settings.domStorageEnabled = true
        webView.setBackgroundColor(android.graphics.Color.TRANSPARENT)
        webView.isVerticalScrollBarEnabled = false
        webView.isHorizontalScrollBarEnabled = false
        webView.overScrollMode = WebView.OVER_SCROLL_NEVER
        webView.addJavascriptInterface(CmHost(id), "CmHost")
        webView.webViewClient = object : WebViewClient() {
            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                val titlesJson = titlesToJson(titles)
                val js = "cmBridge.createEditor(${jsonQuote(id)}, document.getElementById('host'), " +
                    "${jsonQuote(initial)}, ${jsonQuote(titlesJson)})"
                view?.evaluateJavascript(js) {
                    synchronized(pendingOps) {
                        readyIds.add(id)
                        pendingOps.remove(id)?.forEach { op -> runCatching { op() } }
                    }
                    runCatching { onReady?.invoke() }
                }
            }
        }
        webView.loadUrl(CM_EDITOR_URL)
    }

    fun isReady(id: String): Boolean = synchronized(pendingOps) { readyIds.contains(id) }

    fun setText(id: String, text: String) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.setText(${jsonQuote(id)}, ${jsonQuote(text)})", null)
        }
    }

    fun getText(id: String, cb: (String) -> Unit) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.getText(${jsonQuote(id)})", ValueCallback { raw ->
                val parsed = runCatching { JSONTokener(raw).nextValue() }.getOrNull()
                cb(parsed as? String ?: "")
            })
        }
    }

    fun getSelection(id: String, cb: (Int, Int) -> Unit) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.getSelection(${jsonQuote(id)})", ValueCallback { raw ->
                val arr = runCatching { JSONTokener(raw).nextValue() as? JSONArray }.getOrNull()
                if (arr == null) cb(0, 0)
                else cb(arr.optInt(0, 0), arr.optInt(1, 0))
            })
        }
    }

    fun setSelection(id: String, anchor: Int, head: Int) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.setSelection(${jsonQuote(id)}, $anchor, $head)", null)
        }
    }

    /** edge: "start" or "end". Moves the caret there and focuses the editor. */
    fun focus(id: String, edge: String) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.focus(${jsonQuote(id)}, ${jsonQuote(edge)})", null)
        }
    }

    /** Focuses the editor and leaves the selection where it is. */
    fun focusKeepCaret(id: String) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.focusKeepCaret(${jsonQuote(id)})", null)
        }
    }

    fun setEditable(id: String, editable: Boolean) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.setEditable(${jsonQuote(id)}, $editable)", null)
        }
    }

    fun setPlaceholder(id: String, text: String) {
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.setPlaceholder(${jsonQuote(id)}, ${jsonQuote(text)})", null)
        }
    }

    fun setTitles(id: String, titles: Map<String, PageTitle>) {
        val json = titlesToJson(titles)
        withReady(id) { webView ->
            webView.evaluateJavascript("cmBridge.setTitles(${jsonQuote(id)}, ${jsonQuote(json)})", null)
        }
    }

    /**
     * Android-only: measure the rendered content height in CSS px (which equal
     * Compose dp on Android's WebView given the viewport meta in editor_host.html).
     * Used by EditableBlock to size the WebView that hosts the editor.
     */
    fun getContentHeight(id: String, cb: (Float) -> Unit) {
        withReady(id) { webView ->
            webView.evaluateJavascript(
                "(document.getElementById('host')||document.body||document.documentElement).scrollHeight",
                ValueCallback { raw ->
                    val v = raw?.trim()?.toFloatOrNull() ?: 0f
                    cb(v)
                }
            )
        }
    }

    /**
     * Insert [text] at the current selection/caret. Uses getText + getSelection to
     * compute the around-selection text, then setText + setSelection + focusKeepCaret.
     * Used by the special menu's "link to existing page" flow.
     */
    fun insertAtCursor(id: String, text: String) {
        getText(id) { current ->
            getSelection(id) { anchor, head ->
                val lo = minOf(anchor, head).coerceIn(0, current.length)
                val hi = maxOf(anchor, head).coerceIn(0, current.length)
                val newText = current.substring(0, lo) + text + current.substring(hi)
                setText(id, newText)
                val caret = lo + text.length
                setSelection(id, caret, caret)
                focusKeepCaret(id)
            }
        }
    }

    fun destroy(id: String) {
        webViews[id]?.let { webView ->
            runCatching { webView.evaluateJavascript("cmBridge.destroy(${jsonQuote(id)})", null) }
            runCatching { webView.removeJavascriptInterface("CmHost") }
        }
        webViews.remove(id)
        eventHandlers.remove(id)
        synchronized(pendingOps) {
            readyIds.remove(id)
            pendingOps.remove(id)
        }
    }

    private fun withReady(id: String, op: (WebView) -> Unit) {
        val webView = webViews[id] ?: return
        webView.post {
            synchronized(pendingOps) {
                if (readyIds.contains(id)) {
                    runCatching { op(webView) }
                } else {
                    pendingOps.getOrPut(id) { mutableListOf() }.add { runCatching { op(webView) } }
                }
            }
        }
    }

    private fun titlesToJson(titles: Map<String, PageTitle>): String {
        val root = JSONObject()
        for ((k, v) in titles) {
            root.put(k, JSONObject().put("title", v.title).put("disabled", v.disabled))
        }
        return root.toString()
    }

    private fun jsonQuote(s: String): String = JSONObject.quote(s)

    /**
     * One instance per WebView. Dispatches JS-emitted events back to the caller's
     * `onEvent`; returns true for consumed key events so `editor_host.html` can
     * call `preventDefault()` synchronously.
     */
    private class CmHost(private val expectedId: String) {
        @JavascriptInterface
        fun postEvent(detailJson: String): Boolean {
            val handler = eventHandlers[expectedId] ?: return false
            val obj = runCatching { JSONObject(detailJson) }.getOrNull() ?: return false
            if (obj.optString("id") != expectedId) return false
            val event: CmEvent = when (obj.optString("kind")) {
                "changes" -> {
                    val arr = obj.optJSONArray("changes") ?: JSONArray()
                    val changes = ArrayList<Change>(arr.length())
                    for (i in 0 until arr.length()) {
                        val c = arr.optJSONObject(i) ?: continue
                        changes.add(Change(c.optInt("from"), c.optInt("to"), c.optString("insert")))
                    }
                    CmEvent.Changes(changes, obj.optInt("head"))
                }
                "key" -> CmEvent.Key(
                    key = obj.optString("key"),
                    anchor = obj.optInt("anchor"),
                    head = obj.optInt("head"),
                    docLen = obj.optInt("docLen"),
                    ranges = obj.optInt("ranges"),
                )
                "link_click" -> CmEvent.LinkClick(
                    command = obj.optString("command"),
                    rest = obj.optString("rest"),
                )
                "selection" -> CmEvent.Selection(obj.optInt("head"))
                else -> return false
            }
            return try {
                handler(event)
            } catch (e: Throwable) {
                Log.e(TAG, "onEvent failed for kind=${obj.optString("kind")}", e)
                false
            }
        }
    }
}