"""App-specific UI flows for z.zndroid.

These are the reusable building blocks a future test should lean on. Each
function is a short sequence of primitive UI operations, not a full test.
"""
import time

from appium.webdriver.common.appiumby import AppiumBy
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

from . import ui


WAIT = ui.DEFAULT_TIMEOUT

PREFIX = "__AI_AUTOTEST__"

NEW_PAGE_BUTTON = "+ New Page"
NEW_PAGE_DIALOG_TITLE = "New Page"
CREATE_BUTTON = "Create"
BACK_ARROW = "\u2190"
BODY_PLACEHOLDER = "Type '/' for commands..."
RENAME_DIALOG_TITLE = "Rename Page"
DELETE_DIALOG_TITLE = "Delete Page"


def _row_sibling(element, label, timeout):
    """Find the `label` text in the same Row as `element`.

    Compose's a11y tree for a row is Row > Button > Text(title) and
    Row > TextButton > Text("Admin"/"Rename"/"Delete"). The old helper tried
    `following-sibling::*[@text="Delete"]` and `following::*[@text="Delete"][1]`
    from the title Text. Both are fragile in this tree and neither retried.

    This version walks up to the nearest ancestor that contains Admin/Rename/
    Delete (that's the row), then finds `label` inside it. Retries with a
    fresh query each attempt because a LazyColumn can recycle rows during
    scroll animations, making the reference stale for a moment.
    """
    end = time.time() + timeout
    last_error = None
    while time.time() < end:
        try:
            row = element.find_element(
                AppiumBy.XPATH,
                'ancestor::*[.//*[@text="Admin"] and .//*[@text="Rename"] and .//*[@text="Delete"]][1]',
            )
            labels = row.find_elements(
                AppiumBy.XPATH, f'.//*[@text={ui.xpath_literal(label)}]'
            )
            if labels:
                return labels[0]
        except Exception as exc:
            last_error = exc
        time.sleep(0.2)
    raise RuntimeError(f"Could not find '{label}' button in this row: {last_error}")


def ensure_nav_page(driver, timeout=WAIT):
    """Wait for NavPage. If it never appears, try ONE back press as a last resort."""
    if ui.text_present(driver, NEW_PAGE_BUTTON):
        return
    try:
        ui.wait_for_text(driver, NEW_PAGE_BUTTON, timeout=timeout)
        return
    except Exception:
        pass
    ui.press_back(driver)
    ui.wait_for_text(driver, NEW_PAGE_BUTTON, timeout=timeout)


def cleanup_test_pages(driver, prefix=PREFIX, timeout=WAIT):
    """Delete every page whose title starts with `prefix`. Assumes NavPage is ready."""
    title_xpath = f'//*[starts-with(@text, {ui.xpath_literal(prefix)})]'

    for _ in range(50):
        titles = driver.find_elements(AppiumBy.XPATH, title_xpath)
        if not titles:
            return
        el = titles[0]
        try:
            title_text = el.text
        except Exception:
            title_text = ""
        if not title_text.startswith(prefix):
            return

        delete_btn = _row_sibling(el, "Delete", timeout)
        delete_btn.click()

        confirm_xp = (
            f'//*[@text={ui.xpath_literal(DELETE_DIALOG_TITLE)}]'
            f'/following::*[@text="Delete"][1]'
        )
        confirm = WebDriverWait(driver, timeout).until(
            EC.element_to_be_clickable((AppiumBy.XPATH, confirm_xp))
        )
        confirm.click()

        try:
            ui.wait_text_gone(driver, title_text, timeout=timeout)
        except Exception:
            pass


def create_page(driver, name):
    ui.click_by_text(driver, NEW_PAGE_BUTTON)
    WebDriverWait(driver, WAIT).until(
        EC.presence_of_element_located(
            (AppiumBy.XPATH, f'//*[@text={ui.xpath_literal(NEW_PAGE_DIALOG_TITLE)}]')
        )
    )
    field = WebDriverWait(driver, WAIT).until(
        lambda d: d.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
    )[0]
    field.click()
    ui.type_text(driver, name)

    create_xp = (
        f'//*[@text={ui.xpath_literal(NEW_PAGE_DIALOG_TITLE)}]'
        f'/following::*[@text={ui.xpath_literal(CREATE_BUTTON)}][1]'
    )
    create_btn = WebDriverWait(driver, WAIT).until(
        EC.element_to_be_clickable((AppiumBy.XPATH, create_xp))
    )
    create_btn.click()

    # The dialog closing is the real proof the page was created. If the app
    # errors, the dialog stays up and an ERROR popup appears. Waiting here
    # also gives the async DB write time to fail (and surface its popup)
    # before we give up — so runner.step() will see the popup text.
    try:
        ui.wait_text_gone(driver, NEW_PAGE_DIALOG_TITLE, timeout=15)
    except Exception:
        time.sleep(0.5)
        raise AssertionError(
            f"New Page dialog did not close after clicking Create for {name!r}. "
            f"Page creation almost certainly failed — the error popup text should "
            f"be captured alongside this failure in the report."
        )

    # The app auto-scrolls the nav list to the newly created page.
    # If this fails, either the scroll didn't happen or the page wasn't created.
    try:
        ui.wait_for_text(driver, name, timeout=WAIT)
    except Exception:
        raise AssertionError(
            f"page {name!r} not in nav after creation. "
            f"dialog closed but the name never appeared. "
            f"either the nav-scroll-to-new-page didn't run or the page wasn't persisted. "
            f"check logcat for GlobalPopup or AddBlock errors."
        )


def open_page(driver, title):
    ui.click_by_text(driver, title)
    try:
        WebDriverWait(driver, WAIT).until(
            EC.presence_of_element_located(
                (AppiumBy.XPATH, f'//*[@text={ui.xpath_literal(BACK_ARROW)}]')
            )
        )
    except Exception:
        still_on_nav = ui.text_present(driver, NEW_PAGE_BUTTON)
        raise AssertionError(
            f"open_page({title!r}) did not reach ViewPage "
            f"(still_on_nav={still_on_nav}). "
            f"A screen XML is saved next to the failure screenshot."
        )


def _block_is_empty(el):
    """An empty block reports either '' or the placeholder text, depending
    on how Compose merges the placeholder into the EditText's a11y node."""
    text = (el.text or "").strip()
    return text == "" or text == BODY_PLACEHOLDER


def add_body_block(driver, body_text):
    """Put body_text into a body block.

    Fresh pages have two blocks — a title and an empty body. If the last
    block is empty, fill it. Otherwise split the last block to create a
    new empty one, then fill that.

    Both branches are needed. The first call on a fresh page fills the
    pre-existing body block. Later calls split the last block to add
    another, which is what multiple-body-block tests want.
    """
    WebDriverWait(driver, WAIT).until(
        EC.presence_of_element_located((AppiumBy.XPATH, '//android.widget.EditText'))
    )

    fields = ui.find_edit_fields(driver)
    if len(fields) >= 2 and _block_is_empty(fields[-1]):
        fields[-1].click()
    else:
        fields[-1].click()
        ui.press_move_end(driver)
        ui.press_enter(driver)
        time.sleep(0.3)
        fields = ui.find_edit_fields(driver)
        fields[-1].click()

    ui.type_text(driver, body_text)
    ui.wait_for_text(driver, body_text, timeout=WAIT)

    # The app debounces DB writes by 500 ms. Give it room before navigating.
    time.sleep(1.2)


def go_back_to_nav(driver, expected_title=None):
    ui.click_by_text(driver, BACK_ARROW)
    ensure_nav_page(driver)
    if expected_title is not None:
        ui.wait_for_text(driver, expected_title, timeout=WAIT)


def rename_page(driver, old_name, new_name):
    el = ui.find_by_text(driver, old_name)
    rename_btn = _row_sibling(el, "Rename", timeout=WAIT)
    rename_btn.click()

    WebDriverWait(driver, WAIT).until(
        EC.presence_of_element_located(
            (AppiumBy.XPATH, f'//*[@text={ui.xpath_literal(RENAME_DIALOG_TITLE)}]')
        )
    )
    field = WebDriverWait(driver, WAIT).until(
        lambda d: d.find_elements(AppiumBy.XPATH, '//android.widget.EditText')
    )[0]
    field.click()
    field.clear()
    ui.type_text(driver, new_name)

    confirm_xp = (
        f'//*[@text={ui.xpath_literal(RENAME_DIALOG_TITLE)}]'
        f'/following::*[@text="Rename"][1]'
    )
    confirm = WebDriverWait(driver, WAIT).until(
        EC.element_to_be_clickable((AppiumBy.XPATH, confirm_xp))
    )
    confirm.click()

    ui.wait_for_text(driver, new_name, timeout=WAIT)
    ui.wait_text_gone(driver, old_name, timeout=WAIT)


def delete_page(driver, title):
    el = ui.find_by_text(driver, title)
    delete_btn = _row_sibling(el, "Delete", timeout=WAIT)
    delete_btn.click()

    confirm_xp = (
        f'//*[@text={ui.xpath_literal(DELETE_DIALOG_TITLE)}]'
        f'/following::*[@text="Delete"][1]'
    )
    confirm = WebDriverWait(driver, WAIT).until(
        EC.element_to_be_clickable((AppiumBy.XPATH, confirm_xp))
    )
    confirm.click()

    ui.wait_text_gone(driver, title, timeout=WAIT)
