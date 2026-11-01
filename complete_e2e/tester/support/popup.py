from playwright.sync_api import Page


def popup(page: Page, message: str) -> None:
    escaped = message.replace("\\", "\\\\").replace("'", "\\'")
    page.evaluate(
        f"window.dispatchEvent(new CustomEvent('show_popup', {{ detail: '{escaped}' }}))"
    )
