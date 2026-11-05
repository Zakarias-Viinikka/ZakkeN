import random
import re

from playwright.sync_api import Page, expect

from support.seconds import seconds


def on_setup_page(page: Page) -> bool:
    return page.url.rstrip("/").endswith("/setup")


def setup_page_register(page: Page, name: str) -> None:
    page.fill('[data-testid="username_input"]', name)
    page.click('[data-testid="finish_setup_button"]')
    expect(page).to_have_url(re.compile(r"/Menu$"), timeout=seconds(10))


def bring_up_registered(page: Page, port: int, name_prefix: str, timeout: float = 4.0) -> None:
    """Land the page at /Menu. Handles all three entry states:
       - on /custom_port_setting_page (fresh tab): set port, navigate to /,
         then register.
       - on /setup (e.g. after reset_everything wiped user_id): register.
       - on /Menu (already registered): do nothing.
    """
    # Wait for any in-flight reload to settle first, otherwise page.url can
    # still be the pre-reset URL and we'd wrongly conclude "already on /Menu".
    if "custom_port_setting_page" not in page.url:
        page.wait_for_url(re.compile(r"/(setup|Menu|custom_port_setting_page)$"), timeout=seconds(timeout))

    if "custom_port_setting_page" in page.url:
        expect(page.locator("text=custom port setting page")).to_be_visible(timeout=seconds(timeout))
        page.evaluate(f"window.dispatchEvent(new CustomEvent('set_port', {{ detail: {port} }}))")
        page.evaluate("window.dispatchEvent(new CustomEvent('navigate', { detail: '/' }))")
        page.wait_for_url(re.compile(r"/(setup|Menu)$"), timeout=seconds(timeout))

    if on_setup_page(page):
        name = f"{name_prefix}_{random.randint(100000, 999999)}"
        setup_page_register(page, name)
    else:
        expect(page).to_have_url(re.compile(r"/Menu$"))