"""Yrs blob and every_block_in_existence must agree after every mutation,
including after the app process is killed and restarted.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from support import flows
from support.flows import PREFIX


def test_sync_integrity(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"
    body = f"body_{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Sync after create",
             "yrs and every_block_in_existence agree",
             lambda: run.assert_sync(page_name))

    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add body block", f"body {body!r} visible",
             lambda: flows.add_body_block(run.driver, body))
    run.step("Sync after add",
             "yrs and every_block_in_existence agree",
             lambda: run.assert_sync(page_name))

    run.step("Back to nav", "nav visible",
             lambda: flows.go_back_to_nav(run.driver, page_name))
    run.step("Kill and relaunch app", "app back on nav",
             lambda: _kill_and_relaunch(run))
    run.step("Reopen page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Sync after process kill",
             "yrs and every_block_in_existence agree after reload",
             lambda: run.assert_sync(page_name))


def _kill_and_relaunch(run):
    run.restart_app_and_wait(settle=3.0)
    flows.ensure_nav_page(run.driver, timeout=15)
