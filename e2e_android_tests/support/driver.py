import os
import sys

from appium import webdriver
from appium.options.android import UiAutomator2Options


APPIUM_URL = os.environ.get("APPIUM_URL", "http://127.0.0.1:4723")
APP_PACKAGE = os.environ.get("APP_PACKAGE", "z.zndroid")
APP_ACTIVITY = os.environ.get("APP_ACTIVITY", "z.zndroid.MainActivity")
DEVICE_SERIAL = os.environ.get("DEVICE_SERIAL", "")


def create_driver():
    options = UiAutomator2Options()
    options.platform_name = "Android"
    options.device_name = DEVICE_SERIAL or "android"
    if DEVICE_SERIAL:
        options.udid = DEVICE_SERIAL
    options.app_package = APP_PACKAGE
    options.app_activity = APP_ACTIVITY
    options.no_reset = True
    options.full_reset = False
    options.new_command_timeout = 180
    options.set_capability("uiautomator2ServerInstallTimeout", 180000)
    options.set_capability("uiautomator2ServerLaunchTimeout", 180000)
    options.set_capability("adbExecTimeout", 180000)

    print(
        f"[e2e] opening Appium session on {DEVICE_SERIAL or 'first device'} "
        f"(first run installs helper APKs on the phone; 1-2 min over wireless)",
        file=sys.stderr,
        flush=True,
    )
    driver = webdriver.Remote(APPIUM_URL, options=options)
    caps = getattr(driver, "capabilities", {}) or {}
    print(
        f"[e2e] session ready: udid={caps.get('udid', '?')} "
        f"app={APP_PACKAGE} platform={caps.get('platformVersion', '?')}",
        file=sys.stderr,
        flush=True,
    )
    return driver
