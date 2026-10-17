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


def press_enter(driver):
    driver.press_keycode(66)


def press_back(driver):
    driver.press_keycode(4)


def press_delete(driver):
    driver.press_keycode(67)


def type_text(driver, text):
    driver.execute_script('mobile: type', {'text': text})
