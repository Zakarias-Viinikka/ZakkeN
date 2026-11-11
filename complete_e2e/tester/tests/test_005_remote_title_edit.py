"""Audit finding #6: a title edited on another client must show up in the
menu of a client that never opened the page. The MENU_TICK fix landed on
2026-11-05, so this should PASS and acts as a regression test."""

import re

from playwright.sync_api import expect

from support.blocks import (
    block_with_value,
    caret_to_end,
    open_page_from_menu,
    report_js_errors_on_failure,
    watch_js_errors,
)
from support.client_db import read_titles
from support.fresh_world import (
    count_letters,
    fresh_world_with_three_pages,
    wait_for_new_letter,
)
from support.narrate import announce_result, say_both
from support.popup import assert_no_error_popups
from support.seconds import seconds


def test_remote_title_edit_shows_in_menu_of_a_client_that_never_opened_the_page(
    two_browser_stage,
):
    stage = two_browser_stage
    brave = stage.brave
    edge = stage.edge
    brave_errors = watch_js_errors(brave)
    edge_errors = watch_js_errors(edge)

    with announce_result(stage), report_js_errors_on_failure(brave_errors, edge_errors):
        fresh_world_with_three_pages(stage)
        expect(edge).to_have_url(re.compile(r"/Menu$"))

        say_both(stage, "1/5 Edge stays on the menu and never opens the Grocery list page")
        say_both(stage, "2/5 Brave opens the Grocery list page")
        open_page_from_menu(brave, "Grocery list", 6)

        say_both(stage, "3/5 Brave adds ' v2' to the page title")
        before = count_letters()
        caret_to_end(block_with_value(brave, "Grocery list"))
        brave.keyboard.insert_text(" v2")
        wait_for_new_letter(before, brave)

        say_both(stage, "4/5 server has the new title, watching Edge's menu for 'Grocery list v2'", hold=0.3)
        new_title_in_menu = edge.locator("button.title_button", has_text="Grocery list v2")
        try:
            from support.timeouts import ms
            expect(new_title_in_menu).to_have_count(1, timeout=ms())
        except AssertionError:
            raise AssertionError(
                "Edge's menu never showed the new title 'Grocery list v2'. "
                f"Edge's saved titles are {read_titles(edge)}"
            )

        say_both(stage, "5/5 Edge's menu updated, checking that neither browser showed an error", hold=1.5)
        assert_no_error_popups(brave)
        assert_no_error_popups(edge)
        assert not brave_errors and not edge_errors, (brave_errors, edge_errors)

    stage.close()