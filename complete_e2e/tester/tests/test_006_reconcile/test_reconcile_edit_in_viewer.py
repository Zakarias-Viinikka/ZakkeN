"""Reconcile where the user edits the page inside the diff viewer.

Setup: Brave and Edge both have the three seed pages. Edge goes lonewolf
and edits "Grocery list" to "Grocery list v2" in the page editor. Edge
opens /reconcile and picks "Use mine" — the viewer opens with server on
the left ("Grocery list") and Edge's edited copy on the right
("Grocery list v2"). Edge edits the right side to "Grocery list v3"
inside the viewer, clicks Done, closes, submits, and sends. Both sides
end with "Grocery list v3".

The interesting part vs test_reconcile_use_mine: the change reaches the
server from an edit made INSIDE the viewer, not from the client's page
editor. Exercises the viewer → sketches → letters path end to end."""

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
INTERMEDIATE_TITLE = "Grocery list v2"
FINAL_TITLE = "Grocery list v3"
GROCERY_BLOCK_COUNT = 6


def test_reconcile_edit_in_viewer(two_browser_stage):
    stage = two_browser_stage
    brave, edge = stage.brave, stage.edge
    brave_errors = watch_js_errors(brave)
    edge_errors = watch_js_errors(edge)

    with announce_result(stage), report_js_errors_on_failure(brave_errors, edge_errors):
        say_both(stage, "Setup 1/4: fresh server, both browsers have the three sample pages")
        fresh_world_with_three_pages(stage)

        say_both(stage, "Setup 2/4: Edge goes lonewolf")
        enter_lonewolf(edge)
        wait_for_conn_state(edge, LONEWOLF)

        say_both(stage, "Setup 3/4: Edge edits 'Grocery list' -> 'Grocery list v2' in the page editor")
        open_page_from_menu(edge, ORIGINAL_TITLE, GROCERY_BLOCK_COUNT)
        title_block = block_with_value(edge, ORIGINAL_TITLE)
        caret_to_end(title_block)
        title_block.press_sequentially(" v2")
        wait_until_titles(edge, [INTERMEDIATE_TITLE])

        say_both(stage, "Setup 4/4: Edge opens reconcile and picks 'Use mine'")
        open_reconcile(edge)
        click_chooser(edge, 0)
        pick_side(edge, "Use mine")

        say_both(stage, "Step 1/3: Edge edits the title inside the viewer to 'Grocery list v3'")
        # First right-side textarea is the title row (no LeftOnly rows
        # before it in a normal mismatch). fill() replaces the whole
        # value and fires the `input` event the viewer listens on.
        title_ta = edge.locator(".dv-textarea").first
        title_ta.click()
        title_ta.fill(FINAL_TITLE)

        say_both(stage, "Step 2/3: Edge clicks Done (preview shows an EditBlock for the title), then X to close")
        click_done_in_viewer(edge)
        click_number_box_to_close_viewer(edge)

        say_both(stage, "Step 3/3: Edge submits, reviews, sends")
        click_submit(edge)
        click_send_and_finish(edge)
        wait_for_reconcile_to_finish(edge)
        go_to_menu(edge)

        say_both(stage, "Verify: both clients and the server see 'Grocery list v3'")
        wait_until_titles(edge, [FINAL_TITLE])
        wait_until_server_has_pages(3)
        wait_until_titles(brave, [FINAL_TITLE])

        expected = sorted([FINAL_TITLE, "Work notes", "Ideas"])
        assert sorted(read_titles(edge)) == expected, f"Edge titles: {read_titles(edge)}"
        assert sorted(read_titles(brave)) == expected, f"Brave titles: {read_titles(brave)}"

        assert_no_error_popups(brave)
        assert_no_error_popups(edge)
    stage.close()
