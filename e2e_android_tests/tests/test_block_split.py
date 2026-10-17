"""Press Enter at end of a block; verify it splits into two, sync is clean.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows, ui
from support.flows import PREFIX


def test_block_split(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"

    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Type 'hello' in body block", "'hello' visible",
             lambda: flows.add_body_block(run.driver, "hello"))
    run.step("Press Enter to split", "new empty block appears",
             lambda: _split_current(run.driver))
    run.step("Type 'world' into new block", "'world' visible",
             lambda: _type_in_new_block(run.driver, "world"))
    run.step("Verify three blocks: title + 'hello' + 'world'",
             f"blocks are [{page_name!r}, 'hello', 'world']",
             lambda: _assert_blocks(run.driver, [page_name, "hello", "world"]))
    run.step("Sync check (yrs matches every_block_in_existence)",
             "synced=true", lambda: run.assert_sync(page_name))


def _split_current(driver):
    ui.press_enter(driver)
    time.sleep(0.4)


def _type_in_new_block(driver, text):
    ui.type_text(driver, text)
    time.sleep(0.3)


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
