"""Reconcile with one page that exists on both sides but differs.

Setup: Brave and Edge both have the three seed pages. Edge goes lonewolf
and changes the title of "Grocery list" to "Grocery list v2". Brave stays
connected and does nothing. Edge opens /reconcile and sees exactly one
mismatched page (the chooser has one button).

- Use mine: X produces an EditBlock letter. Submit shows it in the
  review. Send sends it. Edge wipes and re-pulls so it ends with the new
  title; Brave receives it live from the server.
- Use server's: X produces zero letters (kept side == server). Submit
  shows nothing to send. Send does nothing. Edge wipes and re-pulls so
  its edit is gone; Brave never sees it.

The merged path (editing inside the viewer) is a separate test."""

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
    click_chooser,
    click_done_in_viewer,
    click_number_box_to_close_viewer,
    click_send_and_finish,
    click_submit,
    go_to_menu,
    open_reconcile,
    pick_side,
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

        say_both(stage, "Step 1/4: Edge opens the one mismatched page")
        click_chooser(edge, 0)

        say_both(stage, "Step 2/4: Edge picks 'Use mine' — viewer opens with server on left, edge on right")
        pick_side(edge, "Use mine")

        say_both(stage, "Step 3/4: Edge clicks Done (preview strip shows the EditBlock), then X to close")
        click_done_in_viewer(edge)
        click_number_box_to_close_viewer(edge)

        say_both(stage, "Step 4/4: Edge submits, reviews, sends")
        click_submit(edge)
        click_send_and_finish(edge)
        wait_for_reconcile_to_finish(edge)
        go_to_menu(edge)

        say_both(stage, "Verify: both browsers and the server should have the new title")
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

        say_both(stage, "Step 1/4: Edge opens the one mismatched page")
        click_chooser(edge, 0)

        say_both(stage, "Step 2/4: Edge picks 'Use server's' — viewer opens with edge on left, server on right")
        pick_side(edge, "Use server's")

        say_both(stage, "Step 3/4: Edge clicks Done (no letters to send), then X to close")
        click_done_in_viewer(edge)
        click_number_box_to_close_viewer(edge)

        say_both(stage, "Step 4/4: Edge submits, reviews (no letters shown), sends")
        click_submit(edge)
        click_send_and_finish(edge)
        wait_for_reconcile_to_finish(edge)
        go_to_menu(edge)

        say_both(stage, "Verify: Edge's edit is gone, nothing reached the server or Brave")
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