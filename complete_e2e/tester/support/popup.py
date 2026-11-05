from playwright.sync_api import Page


def popup(page: Page, message: str) -> None:
    escaped = message.replace("\\", "\\\\").replace("'", "\\'")
    page.evaluate(
        f"window.dispatchEvent(new CustomEvent('show_popup', {{ detail: '{escaped}' }}))"
    )

def assert_no_error_popups(page: Page) -> None:
    raw = page.evaluate("window.__popup_log__ || []")
    errors = [e for e in raw if e.get("kind") == "error"]
    if errors:
        raise AssertionError(
            f"error popups fired: {[e['message'] for e in errors]}"
        )


def clear_popup_log(page: Page) -> None:
    page.evaluate("window.__popup_log__ = []")
