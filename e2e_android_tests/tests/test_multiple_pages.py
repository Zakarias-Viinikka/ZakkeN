"""Create several pages, delete the middle one, verify the others survive.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy

from support import flows
from support.flows import PREFIX


def test_multiple_pages(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    a = f"{PREFIX}{ts}_a"
    b = f"{PREFIX}{ts}_b"
    c = f"{PREFIX}{ts}_c"

    run.step("Create page A", f"{a} visible in nav",
             lambda: flows.create_page(run.driver, a))
    run.step("Create page B", f"{b} visible in nav",
             lambda: flows.create_page(run.driver, b))
    run.step("Create page C", f"{c} visible in nav",
             lambda: flows.create_page(run.driver, c))

    run.step("Verify A, B, C all visible", "all three names present",
             lambda: _all_visible(run.driver, [a, b, c]))

    run.step(f"Delete page B", f"{b} no longer in list",
             lambda: flows.delete_page(run.driver, b))

    run.step("Verify B gone", f"{b} not present",
             lambda: _not_visible(run.driver, b))
    run.step("Verify A and C survived", "A and C still present",
             lambda: _all_visible(run.driver, [a, c]))


def _all_visible(driver, names):
    for n in names:
        ui_els = driver.find_elements(
            AppiumBy.XPATH, f'//*[@text="{n}"]'
        )
        if not ui_els:
            raise AssertionError(f"page {n!r} not visible in nav")


def _not_visible(driver, name):
    els = driver.find_elements(AppiumBy.XPATH, f'//*[@text="{name}"]')
    if els:
        raise AssertionError(f"page {name!r} is still visible in nav")
