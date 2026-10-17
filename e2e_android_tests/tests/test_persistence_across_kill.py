"""Kill the app process, relaunch, verify the page content survived.

This tests more than persistence during a live session: killing the process
discards in-memory Yrs state, so the reload must come from SQLite. If the
mirror was out of sync with the CRDT, the content will be missing or wrong.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from support import flows, ui
from support.flows import PREFIX


def test_persistence_across_kill(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"
    body = f"persisted_{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add body block", f"body text {body!r} visible",
             lambda: flows.add_body_block(run.driver, body))
    run.step("Kill and relaunch app", "app back on nav",
             lambda: _kill_and_relaunch(run))
    run.step("Reopen page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Verify body text survived process kill",
             f"{body!r} still present after kill",
             lambda: ui.wait_for_text(run.driver, body, timeout=10))


def _kill_and_relaunch(run):
    run.restart_app_and_wait(settle=3.0)
    flows.ensure_nav_page(run.driver, timeout=15)
