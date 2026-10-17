"""Typing "/" at the start of an empty block shows the subtle Actions... popup.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from support import flows, ui
from support.flows import PREFIX


def test_slash_popup(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Enter empty body block", "new block appears",
             lambda: _new_block(run.driver))
    run.step("Type '/'", "Actions... popup appears",
             lambda: _type_and_expect_actions(run.driver))
    run.step("Type a letter after '/'", "Actions... popup gone",
             lambda: _type_x_and_expect_no_actions(run.driver))


def _new_block(driver):
    ui.press_enter(driver)
    time.sleep(0.4)


def _type_and_expect_actions(driver):
    ui.type_text(driver, "/")
    ui.wait_for_text(driver, "Actions...", timeout=5)


def _type_x_and_expect_no_actions(driver):
    ui.type_text(driver, "x")
    ui.wait_text_gone(driver, "Actions...", timeout=5)
