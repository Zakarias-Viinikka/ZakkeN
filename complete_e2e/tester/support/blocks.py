"""Page-level helpers for tests that look at and type into blocks.
Blocks are found by their text, never by position: DOM order is not
visual order."""

import re
import time
from contextlib import contextmanager

from playwright.sync_api import Locator, Page, expect

from support.seconds import seconds

BLOCKS = "textarea.ps-content"


def watch_js_errors(page: Page) -> list[str]:
    """Collects uncaught page errors and wasm panics for this page. The
    shared `page` fixture does this for itself, but stage pages are not it."""
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(f"pageerror: {e}"))

    def on_console(msg):
        if msg.type == "error" and "panicked" in msg.text:
            errors.append(f"panic: {msg.text}")

    page.on("console", on_console)
    return errors


def _plain_crash_sentence(errors: list[str]) -> str:
    for err in errors:
        if err.startswith("panic:"):
            lines = err.splitlines()
            what = lines[1].strip() if len(lines) > 1 else "unknown reason"
            return f"The app crashed ({what})."
    first = errors[0].splitlines()[0] if errors else "unknown"
    return f"The app hit an error ({first})."


@contextmanager
def report_js_errors_on_failure(*error_lists):
    try:
        yield
    except AssertionError as e:
        seen = [err for errors in error_lists for err in errors]
        if seen:
            raise AssertionError(
                f"{_plain_crash_sentence(seen)} {e}\njs errors seen: {seen}"
            ) from e
        raise


def open_page_from_menu(page: Page, title: str, block_count: int) -> None:
    button = page.locator("button.title_button", has_text=title)
    expect(button).to_have_count(1, timeout=seconds(5))
    button.click()
    expect(page).to_have_url(re.compile(r"/MainContent/"), timeout=seconds(5))
    expect(page.locator(BLOCKS)).to_have_count(block_count, timeout=seconds(5))


def reload_and_wait_for_blocks(page: Page, block_count: int) -> None:
    page.reload()
    expect(page.locator(BLOCKS)).to_have_count(block_count, timeout=seconds(10))


def block_values(page: Page) -> list[str]:
    return page.eval_on_selector_all(BLOCKS, "els => els.map(e => e.value)")


def block_with_value(page: Page, value: str) -> Locator:
    values = block_values(page)
    if value not in values:
        raise AssertionError(f"no block with text {value!r}, blocks are {values}")
    return page.locator(BLOCKS).nth(values.index(value))


def wait_for_value_in_blocks(page: Page, value: str, timeout_secs: float = 3.0) -> list[str]:
    """Polls until some block has exactly `value`. Returns the last list of
    block texts either way, so the caller can assert and print it."""
    deadline = time.time() + timeout_secs
    values = block_values(page)
    while time.time() < deadline and value not in values:
        time.sleep(0.1)
        values = block_values(page)
    return values


def select_range(block: Locator, start: int, end: int) -> None:
    block.evaluate(
        "(el, r) => { el.focus(); el.setSelectionRange(r[0], r[1]); }", [start, end]
    )


def caret_to_end(block: Locator) -> None:
    block.evaluate(
        "el => { el.focus(); const n = el.value.length; el.setSelectionRange(n, n); }"
    )
