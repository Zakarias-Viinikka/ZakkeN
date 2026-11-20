"""Low-level UI primitives for Appium + UiAutomator2 against a Compose app.

Nothing in here knows about z.zndroid specifically. App-specific flows live
in support/flows.py.
"""
from __future__ import annotations

import time

from appium.webdriver.common.appiumby import AppiumBy
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait


DEFAULT_TIMEOUT = 3


def xpath_literal(value: str) -> str:
    if '"' not in value:
        return f'"{value}"'
    if "'" not in value:
        return f"'{value}'"
    parts = value.split('"')
    return "concat(" + ", '\"', ".join(f'"{p}"' for p in parts) + ")"


def find_by_text(driver, text, timeout=DEFAULT_TIMEOUT):
    xpath = f'//*[@text={xpath_literal(text)}]'
    return WebDriverWait(driver, timeout).until(
        EC.presence_of_element_located((AppiumBy.XPATH, xpath))
    )


def click_by_text(driver, text, timeout=DEFAULT_TIMEOUT):
    element = find_by_text(driver, text, timeout)
    element.click()
    return element


def wait_for_text(driver, text, timeout=DEFAULT_TIMEOUT):
    return find_by_text(driver, text, timeout)


def wait_text_gone(driver, text, timeout=DEFAULT_TIMEOUT):
    xpath = f'//*[@text={xpath_literal(text)}]'
    WebDriverWait(driver, timeout).until_not(
        EC.presence_of_element_located((AppiumBy.XPATH, xpath))
    )


def text_present(driver, text) -> bool:
    xpath = f'//*[@text={xpath_literal(text)}]'
    return bool(driver.find_elements(AppiumBy.XPATH, xpath))


def find_edit_fields(driver):
    return driver.find_elements(AppiumBy.XPATH, '//android.widget.EditText')


def press_move_end(driver):
    driver.press_keycode(123)


def press_move_home(driver):
    driver.press_keycode(122)


def press_enter(driver):
    driver.press_keycode(66)


def press_back(driver):
    driver.press_keycode(4)


def press_delete(driver):
    driver.press_keycode(67)


def _focused_element(driver):
    """The EditText that currently has input focus, or None."""
    try:
        return driver.find_element(
            AppiumBy.XPATH, '//android.widget.EditText[@focused="true"]'
        )
    except Exception:
        return None


def _focused_text(driver):
    el = _focused_element(driver)
    return (el.text if el else "") or ""


def _type_raw(driver, text):
    driver.execute_script('mobile: type', {'text': text})


def _clear_focused(driver):
    """Empty the focused field by backspacing one char at a time.

    element.clear() is reported to throw InvalidElementStateException on
    Compose TextFields, and mobile: clear doesn't exist. Key events are
    the reliable path.
    """
    el = _focused_element(driver)
    if not el:
        return
    current = el.text or ""
    press_move_end(driver)
    for _ in range(len(current)):
        press_delete(driver)
    time.sleep(0.15)


def type_text(driver, text):
    """Type into whatever field has focus, with readback verification.

    Appium's `mobile: type` drops characters on some Samsung devices
    (appium-uiautomator2-driver issue #935): the soft keyboard appears and
    disappears during the call, the field loses focus for a frame or two,
    and a character is swallowed. Symptom is text like 'helo' where you
    sent 'hello', with no error.

    This version types, reads the field back, and retries if the result
    doesn't match what was asked for. The fast path is a single type call;
    the retry path only runs when the bug actually fires.
    """
    if not text:
        return

    before = _focused_text(driver)
    expected = before + text

    _type_raw(driver, text)
    if _focused_text(driver) == expected:
        return

    for _ in range(3):
        _clear_focused(driver)
        _type_raw(driver, expected)
        if _focused_text(driver) == expected:
            return
        time.sleep(0.15)

    raise AssertionError(
        f"typing {text!r} into focused field failed after retries. "
        f"field was {before!r}, expected {expected!r}, got {_focused_text(driver)!r}"
    )


# GlobalPopup (the lab toast stack) exposes each toast as ONE node whose
# content-desc is "popup|<error|info>|<message>" (see GlobalPopup.kt,
# clearAndSetSemantics). Tapping the node dismisses the toast.
# POPUP_ACTION_LABEL is the legacy Snackbar button text, kept only so old
# imports still resolve.
POPUP_ACTION_LABEL = "Dismiss"
POPUP_DESC_PREFIX = "popup|"
ERROR_PREFIX = "ERROR: "
_POPUP_XPATH = f'//*[starts-with(@content-desc, {xpath_literal(POPUP_DESC_PREFIX)})]'


def _popup_nodes(driver):
    return driver.find_elements(AppiumBy.XPATH, _POPUP_XPATH)


def popup_present(driver) -> bool:
    return bool(_popup_nodes(driver))


def get_popup(driver):
    """Return (message, is_error) if a GlobalPopup toast is visible, else None."""
    nodes = _popup_nodes(driver)
    if not nodes:
        return None
    try:
        desc = nodes[0].get_attribute("content-desc") or ""
    except Exception:
        return ("(popup visible, message not readable)", False)
    parts = desc.split("|", 2)
    if len(parts) != 3 or not parts[2].strip():
        return ("(popup visible, message not readable)", False)
    return (parts[2].strip(), parts[1] == "error")


def get_popup_message(driver):
    """Return the popup message if a toast is visible, else None."""
    popup = get_popup(driver)
    return popup[0] if popup else None


def dismiss_popup(driver) -> bool:
    dismissed = False
    for n in _popup_nodes(driver):
        try:
            n.click()
            dismissed = True
        except Exception:
            continue
    return dismissed