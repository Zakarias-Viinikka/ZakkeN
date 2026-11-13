"""Helpers for the /reconcile page. The post-port flow:

    open_reconcile
    -> chooser row (numbered buttons)
    -> pick screen ("Use mine" / "Use server's")
    -> inline viewer (Done / X)
    -> Submit
    -> review panel
    -> Send and finish
    -> reload, back to Connected

Client-only pages (no server copy) don't appear in the chooser; they go
straight to the review panel on Submit as `CreateNewPage` letters.
"""

import re

from playwright.sync_api import Page, expect

from support.timeouts import ms


def open_reconcile(page: Page) -> None:
    page.evaluate(
        "window.dispatchEvent(new CustomEvent('navigate', { detail: '/reconcile' }))"
    )
    expect(page).to_have_url(re.compile(r"/reconcile$"), timeout=ms())
    expect(page.locator("h1", has_text="Reconcile")).to_be_visible(timeout=ms())


def click_chooser(page: Page, index: int = 0) -> None:
    """Click the numbered chooser button at 0-based `index`. Opens the
    pick screen (or the viewer if a default side is set)."""
    btn = page.locator(".rp-chooser-btn").nth(index)
    expect(btn).to_be_visible(timeout=ms())
    btn.click(timeout=ms())


def pick_side(page: Page, label: str) -> None:
    """`label` is "Use mine" or "Use server's". Opening the viewer is a
    side effect of the pick."""
    btn = page.get_by_role("button", name=label, exact=True)
    expect(btn).to_be_visible(timeout=ms())
    btn.click(timeout=ms())


def click_done_in_viewer(page: Page) -> None:
    """Done produces love letters for the current viewer state and shows
    the preview strip. Viewer stays open."""
    btn = page.get_by_role("button", name="Done", exact=True)
    expect(btn).to_be_visible(timeout=ms())
    btn.click(timeout=ms())


def click_number_box_to_close_viewer(page: Page) -> None:
    """The blue numbered box in the viewer toolbar. Produces love letters
    (same as Done) and closes back to the chooser."""
    btn = page.locator(".rp-host-number-box").first
    expect(btn).to_be_visible(timeout=ms())
    btn.click(timeout=ms())


def click_submit(page: Page) -> None:
    """Submit is on the nothing-selected panel; opens the review."""
    btn = page.get_by_role("button", name="Submit", exact=True)
    expect(btn).to_be_enabled(timeout=ms())
    btn.click(timeout=ms())


def click_send_and_finish(page: Page) -> None:
    """The review panel's confirm button. Triggers verify + send + wipe +
    reload."""
    btn = page.get_by_role("button", name="Send and finish", exact=True)
    expect(btn).to_be_enabled(timeout=ms())
    btn.click(timeout=ms())


_RECONCILE_FINISH_SECS = 10.0


def wait_for_reconcile_to_finish(page: Page, timeout_secs: float = None) -> None:
    """After Send the app reloads. Wait for the client to reach Connected
    again — the signal it adopted the server's drift version."""
    from support.read_conn_state import wait_for_conn_state

    if timeout_secs is None:
        timeout_secs = _RECONCILE_FINISH_SECS
    wait_for_conn_state(page, "Connected", timeout_secs=timeout_secs)


def go_to_menu(page: Page) -> None:
    page.evaluate(
        "window.dispatchEvent(new CustomEvent('navigate', { detail: '/Menu' }))"
    )
    expect(page).to_have_url(re.compile(r"/Menu$"), timeout=ms())


def reconcile_one_mismatched_page(page: Page, side_label: str) -> None:
    """Full flow for a single mismatched page: chooser -> pick side -> X
    (saves letters, closes) -> Submit -> Send and finish -> wait."""
    click_chooser(page, 0)
    pick_side(page, side_label)
    click_number_box_to_close_viewer(page)
    click_submit(page)
    click_send_and_finish(page)
    wait_for_reconcile_to_finish(page)