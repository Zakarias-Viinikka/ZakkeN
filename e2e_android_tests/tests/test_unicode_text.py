"""Emoji and quotes survive the full round-trip through Yrs and SQLite.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from support import flows, ui
from support.flows import PREFIX


def test_unicode_text(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"
    body = f'''emoji 🎉 single 'quote' double "quote" done'''

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add body with emoji + quotes", "text visible",
             lambda: flows.add_body_block(run.driver, body))
    run.step("Verify exact text present before kill",
             "full string visible",
             lambda: ui.wait_for_text(run.driver, body, timeout=10))

    run.step("Kill and relaunch app", "app back on nav",
             lambda: _kill_and_relaunch(run))
    run.step("Reopen page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Verify exact text survived process kill",
             "full string visible after kill",
             lambda: ui.wait_for_text(run.driver, body, timeout=10))


def _kill_and_relaunch(run):
    run.restart_app_and_wait(settle=3.0)
    flows.ensure_nav_page(run.driver, timeout=15)
