import os
import time
import pytest
from playwright.sync_api import sync_playwright


@pytest.fixture(scope="session")
def browser():
    headless = os.environ.get("HEADLESS", "0") == "1"
    with sync_playwright() as p:
        b = p.chromium.launch(headless=headless, args=["--start-maximized"])
        yield b
        b.close()


@pytest.fixture(scope="session")
def page(browser):
    context = browser.new_context(no_viewport=True)
    pg = context.new_page()
    yield pg
    context.close()


@pytest.fixture(scope="session")
def web_app_url():
    return os.environ["WEB_APP_URL"]


@pytest.fixture
def pause():
    """Sleep between test actions so a person can watch. No-op when
    HEADLESS=1."""

    def _pause(seconds: float = 1.0):
        if os.environ.get("HEADLESS", "0") != "1":
            time.sleep(seconds)

    return _pause