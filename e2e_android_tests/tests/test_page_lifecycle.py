"""Create a page, add a body block, reopen, verify it survived.

The one representative scenario. If this passes, the create + add +
persist path works. If it fails, something is broken at a level that
matters to the user.
"""
import time

from support import flows, ui
from support.flows import PREFIX


def test_page_lifecycle(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"
    body_text = f"body_{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add body block", f"body {body_text!r} visible",
             lambda: flows.add_body_block(run.driver, body_text))
    run.step("Back to nav", "nav visible",
             lambda: flows.go_back_to_nav(run.driver, page_name))
    run.step("Reopen page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Verify body persisted",
             f"{body_text!r} still visible after reopen",
             lambda: ui.wait_for_text(run.driver, body_text, timeout=10))
