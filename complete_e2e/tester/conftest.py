import os
import re
import time
import pytest
from playwright.sync_api import sync_playwright


@pytest.fixture(scope="session")
def playwright_instance():
    with sync_playwright() as p:
        yield p


@pytest.fixture(scope="session")
def browser(playwright_instance):
    headless = os.environ.get("HEADLESS", "0") == "1"
    b = playwright_instance.chromium.launch(headless=headless, args=["--start-maximized"])
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
        from support import speed
        speed.sleep(0.2)
    yield pg
    context.close()


@pytest.fixture(autouse=True)
def _no_js_errors_per_test(request):
    if "web_app" not in request.fixturenames and "page" not in request.fixturenames:
        yield
        return
    page = request.getfixturevalue("page")
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
            from support import speed
            speed.sleep(seconds)

    return _pause


@pytest.fixture(autouse=True)
def _speed_multiplier(request):
    from support import speed
    speed.set_current(speed.multiplier_for(request.node.module.__name__))
    yield
    speed.set_current(1.0)

@pytest.fixture
def two_browser_stage(web_app_url, playwright_instance, request):
    """Two Chromium windows driven as Brave and Edge. Both registered with
    the server, both on /Menu. Test code uses stage.brave and stage.edge
    as normal Pages, and stage.make_brave_big / make_edge_big /
    share_screen_with_edge_and_brave to switch which one is visible."""
    from support.two_browser_stage import TwoBrowserStage
    from support.setup_page import bring_up_registered
    from support.popup import popup

    import time as _t

    # If the shared single-browser page from tests 0-2 is already up,
    # close it. Otherwise it sits on screen alongside the two we're
    # about to launch, and keeps a third websocket open to the server.
    try:
        existing = request.getfixturevalue("page")
        existing.context.close()
    except Exception:
        pass

    stage = TwoBrowserStage.launch(playwright_instance, web_app_url)
    port = int(os.environ["SERVER_PORT"])

    # Popups are fired AFTER each page has settled (navigation wipes toasts,
    # so firing before a navigate is wasted). Numbered so the sequence is
    # obvious when several stack.
    bring_up_registered(stage.brave, port, "e2e_brave")
    popup(stage.brave, "1/2: brave ready - registering edge")
    popup(stage.edge, "1/2: brave ready - registering edge")
    _t.sleep(1.5)

    bring_up_registered(stage.edge, port, "e2e_edge")
    popup(stage.brave, "2/2: both ready - starting test")
    popup(stage.edge, "2/2: both ready - starting test")
    _t.sleep(1.5)

    yield stage
    # deliberately not closing: leave both browsers open for inspection
