"""Read the web app's CONN_STATE via the exposed `read_conn_state`
window event. Returns the Debug string of the `ConnState` enum."""

import time

from playwright.sync_api import Page

from support.timeouts import secs as default_secs


def read_conn_state(page: Page, timeout_secs: float = default_secs()) -> str:
    # While the app reloads after Finish reconcile, `page.evaluate` throws
    # "Execution context was destroyed". That just means the page is
    # navigating; retry until the context comes back.
    deadline = time.time() + timeout_secs
    last_err = None
    while time.time() < deadline:
        try:
            page.evaluate("window.__conn_state__ = ''")
            page.evaluate("window.dispatchEvent(new CustomEvent('read_conn_state'))")
        except Exception as e:
            last_err = e
            time.sleep(0.05)
            continue
        while time.time() < deadline:
            try:
                value = page.evaluate("window.__conn_state__ || ''")
            except Exception as e:
                last_err = e
                break
            if value:
                return value
            time.sleep(0.05)
    if last_err is not None:
        raise AssertionError(f"read_conn_state: {last_err}")
    raise AssertionError("read_conn_state: no value written to window.__conn_state__")


def wait_for_conn_state(page: Page, wanted, timeout_secs: float = default_secs()) -> str:
    """`wanted` is either one string or an iterable of acceptable strings.

    During a page reload, `read_conn_state` raises because the JS context
    is destroyed mid-navigation. Treat that as "not yet", not as failure."""
    wanted_set = {wanted} if isinstance(wanted, str) else set(wanted)
    deadline = time.time() + timeout_secs
    last = ""
    while time.time() < deadline:
        try:
            last = read_conn_state(page, timeout_secs=min(0.5, timeout_secs))
        except AssertionError:
            time.sleep(0.05)
            continue
        if last in wanted_set:
            return last
        time.sleep(0.1)
    raise AssertionError(f"conn_state never reached any of {wanted_set}, last saw {last!r}")
