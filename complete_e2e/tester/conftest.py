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
    pg.on("load", lambda: print(f"### PAGE LOAD: {pg.url}"))
    pg.on("framenavigated", lambda f: print(f"### NAV: {f.url}"))
    pg.js_errors = []
    def on_pageerror(e):
        print(f"### PAGEERROR: {e}")
        pg.js_errors.append(str(e))
    pg.on("pageerror", on_pageerror)
    pg.on("console", lambda m: print(f"### CONSOLE {m.type}: {m.text}"))
    if os.environ.get("HEADLESS", "0") != "1":
        time.sleep(1)
    yield pg
    context.close()


@pytest.fixture(autouse=True)
def _no_js_errors_per_test(page, request):
    start = len(page.js_errors)
    yield
    new = page.js_errors[start:]
    if new:
        raise AssertionError(f"JS errors during {request.node.name}: {new}")


@pytest.fixture(scope="session")
def web_app(page, web_app_url):
    """The page, ready to run tests against. Lands on /custom_port_setting_page,
    sets the server port, then navigates to / so SetupGuard can do its thing."""
    from playwright.sync_api import expect
    from support.seconds import seconds

    page.goto(web_app_url)
    expect(page.locator("text=custom port setting page")).to_be_visible(timeout=seconds(2))
    port = int(os.environ["SERVER_PORT"])
    page.evaluate(f"window.dispatchEvent(new CustomEvent('set_port', {{ detail: {port} }}))")
    page.evaluate("window.dispatchEvent(new CustomEvent('navigate', { detail: '/' }))")
    page.evaluate(
        f"window.dispatchEvent(new CustomEvent('show_popup', {{ detail: 'connecting to port {port} and registering' }}))"
    )
    return page


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