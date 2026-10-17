"""Edit the title block from inside the page; nav should reflect the new title.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows, ui
from support.flows import PREFIX


def test_title_editing(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    original = f"{PREFIX}{ts}"
    edited = f"{original}_v2"

    run.step("Create page", f"{original} visible in nav",
             lambda: flows.create_page(run.driver, original))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, original))
    run.step("Append suffix to title block",
             f"title block now reads {edited!r}",
             lambda: _append_to_title(run.driver, "_v2"))
    run.step("Back to nav", f"nav shows {edited!r}",
             lambda: flows.go_back_to_nav(run.driver, edited))


def _append_to_title(driver, suffix):
    els = driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
    if not els:
        raise AssertionError("no EditText found on page")
    els[0].click()
    ui.press_move_end(driver)
    ui.type_text(driver, suffix)
    # debounce is 500ms; wait for the flush to reach the DB
    time.sleep(1.5)
