from playwright.sync_api import expect


def test_show_popup(web_app, pause):
    web_app.evaluate(
        "window.dispatchEvent(new CustomEvent('show_popup', { detail: 'hi from tester' }))"
    )
    pause(2)
    expect(web_app.locator(".pp-toast-title").last).to_have_text("hi from tester")