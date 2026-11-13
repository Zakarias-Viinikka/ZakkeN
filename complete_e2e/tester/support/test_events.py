"""Fire-and-forget window events that the app exposes for the tester.
None of these wait for anything; they just dispatch and return."""

from playwright.sync_api import Page


def wipe_server(page: Page) -> None:
    page.evaluate("window.dispatchEvent(new CustomEvent('wipe_server'))")


def reset_everything(page: Page) -> None:
    page.evaluate("window.dispatchEvent(new CustomEvent('reset_everything'))")


def insert_three_pages(page: Page) -> None:
    page.evaluate("window.dispatchEvent(new CustomEvent('insert_three_pages'))")

def enter_lonewolf(page):
    """Same action as the dev-sidebar deliberate-lonewolf button."""
    page.evaluate("window.dispatchEvent(new CustomEvent('enter_lonewolf'))")
