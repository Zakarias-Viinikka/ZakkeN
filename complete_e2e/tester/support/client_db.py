"""Read helpers for the client's local db. Each one fires the app's
matching read event, then polls a window global the app fills in."""

import json
import time

from playwright.sync_api import Page


def read_titles(page: Page) -> list[str]:
    from support import speed
    page.evaluate("window.__read_titles__ = null")
    page.evaluate("window.dispatchEvent(new CustomEvent('read_titles'))")
    deadline = time.time() + speed.budget(4.0)
    while time.time() < deadline:
        raw = page.evaluate("window.__read_titles__")
        if raw is not None:
            return json.loads(raw)
        time.sleep(0.1)
    raise AssertionError("read_titles did not populate window.__read_titles__ in time")

def read_client_state(page: Page) -> dict:
    page.evaluate("window.__client_state__ = null")
    page.evaluate("window.dispatchEvent(new CustomEvent('read_client_state'))")
    deadline = time.time() + 4
    while time.time() < deadline:
        raw = page.evaluate("window.__client_state__")
        if raw is not None:
            return json.loads(raw)
        time.sleep(0.1)
    raise AssertionError("read_client_state did not populate in 4s")
