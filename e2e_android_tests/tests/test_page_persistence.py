"""Page lifecycle: create, edit, persist across reopen, rename, delete.

This is the one representative scenario. A future test should copy this
shape (steps + prefix + cleanup) rather than inventing a new pattern.
"""
import time

from selenium.common.exceptions import TimeoutException

from support import flows, ui
from support.flows import PREFIX


def test_page_lifecycle(run):
    uniq = time.strftime("%Y%m%d_%H%M%S")
    page_name = f"{PREFIX}{uniq}"
    new_name = f"{page_name}_renamed"
    body_text = f"body_{uniq}"

    # Cleanup and app reset happen in the session fixture before this test runs.
    run.step(
        "Create a new page from the nav list",
        f"Dialog opens, page '{page_name}' is created and visible in the list",
        lambda: flows.create_page(run.driver, page_name),
    )

    run.step(
        f"Open page '{page_name}'",
        "Page view opens (back arrow visible in top bar)",
        lambda: flows.open_page(run.driver, page_name),
    )

    run.step(
        "Add a body block and type text into it",
        f"Body text '{body_text}' appears on the page",
        lambda: flows.add_body_block(run.driver, body_text),
    )

    run.step(
        "Navigate back to the nav list",
        "NavPage is visible and the page row is still listed",
        lambda: flows.go_back_to_nav(run.driver, page_name),
    )

    run.step(
        f"Reopen page '{page_name}' to verify persistence",
        f"Body text '{body_text}' is still present after reopening",
        lambda: _reopen_and_check_body(run.driver, page_name, body_text),
    )

    run.step(
        "Navigate back to the nav list",
        "NavPage is visible and the page row is still listed",
        lambda: flows.go_back_to_nav(run.driver, page_name),
    )

    run.step(
        f"Rename page to '{new_name}'",
        f"NavPage row shows new name '{new_name}'",
        lambda: flows.rename_page(run.driver, page_name, new_name),
    )

    run.step(
        "Delete the test page",
        f"NavPage row '{new_name}' is no longer visible",
        lambda: flows.delete_page(run.driver, new_name),
    )


def _reopen_and_check_body(driver, page_name, body_text):
    flows.open_page(driver, page_name)
    try:
        ui.wait_for_text(driver, body_text, timeout=ui.DEFAULT_TIMEOUT)
    except TimeoutException as exc:
        raise AssertionError(
            f"Body text '{body_text}' not present after reopening the page"
        ) from exc
