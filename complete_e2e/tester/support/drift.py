"""Helpers for the drift-version / lonewolf tests.

The ConnState names are the Debug strings of the app's `ConnState` enum.
The enum variant is `LonewolfAndOffline` (lowercase w), not
`LoneWolfAndOffline`."""

import time

from playwright.sync_api import Page

from support.client_db import read_client_state
from support.read_conn_state import read_conn_state
from support.timeouts import secs

LONEWOLF = ("Lonewolf", "LonewolfAndOffline")


def assert_conn_state_stays(page: Page, allowed, for_secs: float = None) -> None:
    """Fails if the connection state is ever outside `allowed` during the
    window. A negative wait: it has to sit for the whole window to prove
    nothing changed, so it is not padding."""
    if for_secs is None:
        for_secs = secs()
    allowed_set = {allowed} if isinstance(allowed, str) else set(allowed)
    deadline = time.time() + for_secs
    while time.time() < deadline:
        state = read_conn_state(page)
        if state not in allowed_set:
            raise AssertionError(
                f"conn state left {sorted(allowed_set)}, now {state!r}"
            )
        time.sleep(0.1)


def wait_for_stored_drift_version(page: Page, timeout_secs: float = None) -> str:
    """Polls the client's own state until it has a drift version stored."""
    if timeout_secs is None:
        timeout_secs = secs()
    deadline = time.time() + timeout_secs
    last = None
    while time.time() < deadline:
        try:
            last = read_client_state(page).get("drift_version")
        except Exception:
            last = None
        if last:
            return last
        time.sleep(0.1)
    raise AssertionError(
        f"the client never stored a drift version (waited {timeout_secs:.0f}s, last saw {last!r})"
    )
