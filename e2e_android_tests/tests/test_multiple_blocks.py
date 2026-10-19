"""Add several body blocks in a row and verify they all appear in order.

Checks the basic "user types, blocks appear" loop.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows, ui
from support.flows import PREFIX


def test_multiple_blocks(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Add first body block 'one'", "'one' visible",
             lambda: flows.add_body_block(run.driver, "one"))
    run.step("Add second body block 'two'", "'two' visible",
             lambda: _add_after(run.driver, "two"))
    run.step("Add third body block 'three'", "'three' visible",
             lambda: _add_after(run.driver, "three"))
    run.step("All four blocks present in order",
             f"blocks are [{page_name!r}, 'one', 'two', 'three']",
             lambda: _assert_blocks(run.driver, [page_name, "one", "two", "three"]))


def _add_after(driver, text):
    """From the currently focused last block, split with Enter and type."""
    ui.press_enter(driver)
    time.sleep(0.3)
    ui.type_text(driver, text)
    ui.wait_for_text(driver, text, timeout=ui.DEFAULT_TIMEOUT)
    time.sleep(1.2)


def _assert_blocks(driver, expected):
    deadline = time.time() + ui.DEFAULT_TIMEOUT
    while time.time() < deadline:
        els = driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
        texts = [e.text for e in els]
        if texts == expected:
            return
        time.sleep(0.2)
    els = driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
    texts = [e.text for e in els]
    raise AssertionError(f"expected blocks {expected}, got {texts}")
