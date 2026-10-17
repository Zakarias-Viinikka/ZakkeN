"""Backspace at pos 0 of the title block must not remove or damage the title.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows, ui
from support.flows import PREFIX


def test_title_backspace_noop(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add body 'body'", "'body' visible",
             lambda: flows.add_body_block(run.driver, "body"))
    run.step("Focus title, move home, backspace",
             "title unchanged, both blocks intact",
             lambda: _backspace_in_title(run.driver))
    run.step("Verify title text unchanged",
             f"title still reads {page_name!r}",
             lambda: ui.wait_for_text(run.driver, page_name, timeout=10))
    run.step("Verify body block still present", "'body' visible",
             lambda: ui.wait_for_text(run.driver, "body", timeout=10))
    run.step("Verify exactly 2 EditText", "2 EditText on page",
             lambda: _assert_block_count(run.driver, 2))


def _backspace_in_title(driver):
    els = driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
    if not els:
        raise AssertionError("no EditText on page")
    els[0].click()
    ui.press_move_home(driver)
    time.sleep(0.2)
    ui.press_delete(driver)
    time.sleep(0.5)


def _assert_block_count(driver, expected):
    deadline = time.time() + ui.DEFAULT_TIMEOUT
    while time.time() < deadline:
        n = len(driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText'))
        if n == expected:
            return
        time.sleep(0.2)
    n = len(driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText'))
    raise AssertionError(f"expected {expected} EditText, got {n}")
