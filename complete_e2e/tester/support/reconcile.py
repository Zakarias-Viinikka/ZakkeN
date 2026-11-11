"""Helpers for the /reconcile page: open it via the app's own
`navigate` window event (the SPA route, not a URL reload), click
Finish, wait for the reload that follows."""

import re

from playwright.sync_api import Page, expect

from support.timeouts import ms


def open_reconcile(page: Page) -> None:
    page.evaluate(
        "window.dispatchEvent(new CustomEvent('navigate', { detail: '/reconcile' }))"
    )
    expect(page).to_have_url(re.compile(r"/reconcile$"), timeout=ms())
    expect(page.locator("h1", has_text="Reconcile")).to_be_visible(timeout=ms())


def click_finish_reconcile(page: Page) -> None:
    locator = page.get_by_role("button", name="Finish reconcile", exact=True)
    expect(locator).to_be_enabled(timeout=ms())
    locator.click(timeout=ms())


# The Finish click runs the planner (CPU), sends the batch, wipes
# locally, then reloads. The page reload keeps the /reconcile URL; on
# reload the client re-logs-in and reaches Connected. What matters is
# that the state is Connected, not what URL it's on.
_RECONCILE_FINISH_SECS = 10.0


def wait_for_reconcile_to_finish(page: Page, timeout_secs: float = None) -> None:
    """After Finish, the app reloads. Wait for the client to reach
    Connected again — that's the signal it adopted the server's drift
    version."""
    from support.read_conn_state import wait_for_conn_state

    if timeout_secs is None:
        timeout_secs = _RECONCILE_FINISH_SECS
    wait_for_conn_state(page, "Connected", timeout_secs=timeout_secs)