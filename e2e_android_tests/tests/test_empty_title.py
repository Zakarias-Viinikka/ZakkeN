"""Clear a page's title; nav should show "Untitled" for it.

Counts "Untitled" occurrences on nav before and after, so a pre-existing
Untitled page in the user's data doesn't make this test pass or fail
spuriously.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows, ui
from support.flows import PREFIX


def test_empty_title(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{ts}"

    baseline = [0]

    def _count_untitled():
        return len(run.driver.find_elements(
            AppiumBy.XPATH, '//*[@text="Untitled"]'
        ))

    def _record_baseline():
        baseline[0] = _count_untitled()

    def _assert_one_more():
        now = _count_untitled()
        if now != baseline[0] + 1:
            raise AssertionError(
                f"expected Untitled count to increase by 1 "
                f"(baseline={baseline[0]}, now={now})"
            )

    run.step("Record Untitled count on nav", "baseline captured",
             _record_baseline)
    run.step("Create page", f"{page_name} visible in nav",
             lambda: flows.create_page(run.driver, page_name))
    run.step("Open page", "page view visible",
             lambda: flows.open_page(run.driver, page_name))
    run.step("Clear the title block", "title block is empty",
             lambda: _clear_title(run.driver, page_name))
    run.step("Back to nav", "nav visible",
             lambda: flows.go_back_to_nav(run.driver))
    run.step("Untitled count is baseline+1", "one extra Untitled in nav",
             _assert_one_more)


def _clear_title(driver, current_title):
    els = driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
    if not els:
        raise AssertionError("no EditText found on page")
    els[0].click()
    ui.press_move_end(driver)
    for _ in range(len(current_title) + 5):
        ui.press_delete(driver)
    time.sleep(1.5)
