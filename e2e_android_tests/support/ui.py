"""Low-level UI primitives for Appium + UiAutomator2 against a Compose app.

Nothing in here knows about z.zndroid specifically. App-specific flows live
in support/flows.py.
"""
from __future__ import annotations

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


def type_text(driver, text):
    driver.execute_script('mobile: type', {'text': text})


POPUP_ACTION_LABEL = "Dismiss"


def popup_present(driver) -> bool:
    return bool(driver.find_elements(
        AppiumBy.XPATH, f'//*[@text={xpath_literal(POPUP_ACTION_LABEL)}]'
    ))


def get_popup_message(driver):
    """Return the app-popup message if a GlobalPopup snackbar is visible, else None."""
    dismiss = driver.find_elements(
        AppiumBy.XPATH, f'//*[@text={xpath_literal(POPUP_ACTION_LABEL)}]'
    )
    if not dismiss:
        return None
    for xp in (
        'ancestor::*[count(descendant::*[@text]) = 2][1]',
        'ancestor::*[count(descendant::*[@text]) <= 3][1]',
        'ancestor::*[descendant::*[@text="Dismiss"]][1]',
    ):
        try:
            container = dismiss[0].find_element(AppiumBy.XPATH, xp)
            for t in container.find_elements(AppiumBy.XPATH, './/*[@text]'):
                txt = (t.text or '').strip()
                if txt and txt != POPUP_ACTION_LABEL:
                    return txt
        except Exception:
            continue
    return "(popup visible, message not readable)"


def dismiss_popup(driver) -> bool:
    for n in driver.find_elements(
        AppiumBy.XPATH, f'//*[@text={xpath_literal(POPUP_ACTION_LABEL)}]'
    ):
        try:
            n.click()
            return True
        except Exception:
            continue
    return False


ERROR_PREFIX = "ERROR: "


def get_popup(driver):
    """Return (message, is_error) if a GlobalPopup is visible, else None.

    is_error is True when the visible text starts with ERROR_PREFIX, which
    GlobalPopupManager.showError() prepends.
    """
    dismiss = driver.find_elements(
        AppiumBy.XPATH, f'//*[@text={xpath_literal(POPUP_ACTION_LABEL)}]'
    )
    if not dismiss:
        return None
    text = None
    for xp in (
        'ancestor::*[count(descendant::*[@text]) = 2][1]',
        'ancestor::*[count(descendant::*[@text]) <= 3][1]',
        'ancestor::*[descendant::*[@text="Dismiss"]][1]',
    ):
        try:
            container = dismiss[0].find_element(AppiumBy.XPATH, xp)
            for t in container.find_elements(AppiumBy.XPATH, './/*[@text]'):
                txt = (t.text or '').strip()
                if txt and txt != POPUP_ACTION_LABEL:
                    text = txt
                    break
        except Exception:
            continue
        if text:
            break
    if text is None:
        return ("(popup visible, message not readable)", False)
    return (text, text.startswith(ERROR_PREFIX))
