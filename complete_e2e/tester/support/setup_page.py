import re

from playwright.sync_api import Page, expect

from support.seconds import seconds


def on_setup_page(page: Page) -> bool:
    return page.url.rstrip("/").endswith("/setup")


def setup_page_register(page: Page, name: str) -> None:
    page.fill('[data-testid="username_input"]', name)
    page.click('[data-testid="finish_setup_button"]')
    expect(page).to_have_url(re.compile(r"/Menu$"), timeout=seconds(10))
