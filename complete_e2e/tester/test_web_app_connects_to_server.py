from playwright.sync_api import expect


def test_web_app_loads_without_js_error(page, web_app_url, pause):
    errors = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.goto(web_app_url)
    pause(3)
    assert not errors, f"page errors: {errors}"


def test_show_popup(page, web_app_url, pause, browser):
    page.on("crash", lambda: print("### PAGE CRASHED"))
    page.on("close", lambda pg: print(f"### PAGE CLOSED: {pg.url}"))
    page.on("pageerror", lambda e: print(f"### PAGE ERROR: {e}"))
    browser.on("disconnected", lambda b: print("### BROWSER DISCONNECTED"))
    page.goto(web_app_url)
    pause(2)
    page.evaluate(
        "window.dispatchEvent(new CustomEvent('show_popup', { detail: 'hi from tester' }))"
    )
    pause(2)
    expect(page.locator(".pp-toast-title")).to_have_text("hi from tester")