"""Backspace at pos 0 of an empty body block merges it into the previous one.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows, ui
from support.flows import PREFIX


def test_block_empty_removal(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add body block 'x'", "'x' visible",
             lambda: flows.add_body_block(run.driver, "x"))
    run.step("Verify 3 blocks temporarily (title + 'x' + placeholder)",
             "3 EditText on page",
             lambda: _assert_block_count(run.driver, 3))
    run.step("Backspace to empty the block", "block empty",
             lambda: _empty_block(run.driver))
    run.step("Backspace at pos 0 to remove it",
             "back to 2 blocks (title + placeholder)",
             lambda: _backspace_at_start(run.driver))
    run.step("Verify 2 EditText remain",
             "2 EditText on page",
             lambda: _assert_block_count(run.driver, 2))
    run.step("Verify body placeholder ghost text reappeared",
             "placeholder text visible",
             lambda: ui.wait_for_text(run.driver, flows.BODY_PLACEHOLDER, timeout=5))
    run.step("Sync check", "synced=true", lambda: run.assert_sync(page_name))


def _empty_block(driver):
    ui.press_delete(driver)
    time.sleep(0.6)


def _backspace_at_start(driver):
    ui.press_move_home(driver)
    time.sleep(0.2)
    ui.press_delete(driver)
    time.sleep(0.8)


def _assert_block_count(driver, expected):
    deadline = time.time() + ui.DEFAULT_TIMEOUT
    while time.time() < deadline:
        n = len(driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText'))
        if n == expected:
            return
        time.sleep(0.2)
    n = len(driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText'))
    raise AssertionError(f"expected {expected} EditText, got {n}")
