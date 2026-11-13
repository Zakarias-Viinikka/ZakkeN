"""Reconcile with one page that exists on both sides but differs.

Setup: Brave and Edge both have the three seed pages. Edge goes lonewolf
and changes the title of "Grocery list" to "Grocery list v2". Brave stays
connected and does nothing. Edge opens /reconcile and sees exactly one
card.

- use mine: Finish sends the edit to the server. Edge wipes itself and
  re-pulls, so it ends with the new title. Brave receives it live.
- use server's: Finish sends nothing. Edge wipes itself and re-pulls, so
  its edit is gone. Brave never sees it.

The merged path (diff overlay) is a separate test."""

from support.blocks import (
    block_with_value,
    caret_to_end,
    open_page_from_menu,
    report_js_errors_on_failure,
    watch_js_errors,
)
from support.client_db import read_titles
from support.drift import LONEWOLF
from support.db import wait_until_server_has_pages
from support.fresh_world import (
    SEED_TITLES,
    count_letters,
    fresh_world_with_three_pages,
    wait_until_titles,
)
from support.narrate import announce_result, say_both
from support.popup import assert_no_error_popups
from support.read_conn_state import wait_for_conn_state
from support.reconcile import (
    click_finish_reconcile,
    go_to_menu,
    open_reconcile,
    pick_on_only_card,
    wait_for_reconcile_to_finish,
)
from support.test_events import enter_lonewolf

ORIGINAL_TITLE = "Grocery list"
EDITED_TITLE = "Grocery list v2"
GROCERY_BLOCK_COUNT = 6


def _edge_edits_title_while_lonewolf_then_opens_reconcile(stage) -> None:
    edge = stage.edge

    say_both(stage, "Setup 1/4: fresh server, both browsers have the three sample pages")
    fresh_world_with_three_pages(stage)

    say_both(stage, "Setup 2/4: Edge goes lonewolf (cut off from the server on purpose)")
    enter_lonewolf(edge)
    wait_for_conn_state(edge, LONEWOLF)

    say_both(stage, "Setup 3/4: Edge adds ' v2' to the title of Grocery list")
    open_page_from_menu(edge, ORIGINAL_TITLE, GROCERY_BLOCK_COUNT)
    title_block = block_with_value(edge, ORIGINAL_TITLE)
    caret_to_end(title_block)
    title_block.press_sequentially(" v2")
    # Titles come from the local db, so this also proves the edit was saved
    # before reconcile reads the local pages.
    wait_until_titles(edge, [EDITED_TITLE])

    say_both(stage, "Setup 4/4: Edge opens the reconcile page")
    open_reconcile(edge)


def test_reconcile_use_mine(two_browser_stage):
    stage = two_browser_stage
    brave, edge = stage.brave, stage.edge
    brave_errors = watch_js_errors(brave)
    edge_errors = watch_js_errors(edge)

    with announce_result(stage), report_js_errors_on_failure(brave_errors, edge_errors):
        _edge_edits_title_while_lonewolf_then_opens_reconcile(stage)

        say_both(stage, "Step 1/3: Edge picks 'use mine' for the one page that differs")
        pick_on_only_card(edge, "use mine", "mine")

        say_both(stage, "Step 2/3: Edge finishes reconcile")
        click_finish_reconcile(edge)
        wait_for_reconcile_to_finish(edge)
        go_to_menu(edge)

        say_both(stage, "Step 3/3: both browsers and the server should have the new title")
        wait_until_titles(edge, [EDITED_TITLE])
        wait_until_server_has_pages(3)
        wait_until_titles(brave, [EDITED_TITLE])

        expected = sorted([EDITED_TITLE, "Work notes", "Ideas"])
        assert sorted(read_titles(edge)) == expected, f"Edge titles: {read_titles(edge)}"
        assert sorted(read_titles(brave)) == expected, f"Brave titles: {read_titles(brave)}"

        assert_no_error_popups(brave)
        assert_no_error_popups(edge)
    stage.close()


def test_reconcile_use_server(two_browser_stage):
    stage = two_browser_stage
    brave, edge = stage.brave, stage.edge
    brave_errors = watch_js_errors(brave)
    edge_errors = watch_js_errors(edge)

    with announce_result(stage), report_js_errors_on_failure(brave_errors, edge_errors):
        _edge_edits_title_while_lonewolf_then_opens_reconcile(stage)
        letters_before = count_letters()

        say_both(stage, "Step 1/3: Edge picks 'use server's' for the one page that differs")
        pick_on_only_card(edge, "use server's", "server")

        say_both(stage, "Step 2/3: Edge finishes reconcile")
        click_finish_reconcile(edge)
        wait_for_reconcile_to_finish(edge)
        go_to_menu(edge)

        say_both(stage, "Step 3/3: Edge's edit is gone, nothing reached the server or Brave")
        wait_until_titles(edge, SEED_TITLES)
        wait_until_server_has_pages(3)

        assert sorted(read_titles(edge)) == sorted(SEED_TITLES), (
            f"Edge titles: {read_titles(edge)}"
        )
        assert sorted(read_titles(brave)) == sorted(SEED_TITLES), (
            f"Brave titles: {read_titles(brave)}"
        )
        assert count_letters() == letters_before, (
            f"server letters went from {letters_before} to {count_letters()}, "
            "but 'use server's' should send nothing"
        )

        assert_no_error_popups(brave)
        assert_no_error_popups(edge)
    stage.close()
