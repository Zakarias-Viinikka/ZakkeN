"""Popups a human can follow. Fire a message on one or both browsers, then
hold for a moment (scaled by the per-test speed) so it can be read.

Every narration message starts with a digit, "Setup", "Step" or "TEST", so
app_said() can tell them apart from popups the app itself showed."""

import re
from contextlib import contextmanager

from support import speed
from support.popup import popup

_NARRATION = re.compile(r"^(\d|Setup|Step|TEST)")


def say(page, message: str, hold: float = 1.0) -> None:
    popup(page, message)
    speed.sleep(hold)


def say_both(stage, message: str, hold: float = 1.0) -> None:
    popup(stage.brave, message)
    popup(stage.edge, message)
    speed.sleep(hold)


def app_said(page) -> str:
    """Popups the app itself showed on this page (not our narration), as a
    sentence to append to a failure message. Empty if there were none."""
    if page is None:
        return ""
    try:
        log = page.evaluate("window.__popup_log__ || []")
    except Exception:
        return ""
    messages = [
        e.get("message", "") for e in log if not _NARRATION.match(e.get("message", ""))
    ]
    if not messages:
        return ""
    return " The app showed: " + " | ".join(messages[-3:])


@contextmanager
def announce_result(stage):
    """Wrap a whole test. Puts TEST PASSED or TEST FAILED on both screens, so
    a failure is never silent."""
    try:
        yield
    except BaseException as e:
        reason = (str(e).strip().splitlines() or ["unknown reason"])[0]
        if len(reason) > 220:
            reason = reason[:220] + "..."
        try:
            say_both(stage, f"TEST FAILED: {reason}", hold=4.0)
        except Exception:
            pass
        raise
    else:
        say_both(stage, "TEST PASSED", hold=1.0)