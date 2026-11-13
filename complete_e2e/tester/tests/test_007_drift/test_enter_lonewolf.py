"""Tests-to-write #2, the parts that don't need the browser "offline"
event.

Edge is connected. Entering lonewolf puts it in `Lonewolf` and it stays
there. Entering lonewolf a second time must not change that (in
particular it must not become `LonewolfAndOffline`). An edit made while
lonewolf stays local: nothing reaches the server and Brave doesn't see
it.

Not written yet: reload, fire a synthetic offline event, enter lonewolf,
expect `LonewolfAndOffline`. Needs a look at what the app listens to."""

import time

from support.blocks import (
    block_with_value,
    caret_to_end,
    open_page_from_menu,
    report_js_errors_on_failure,
    watch_js_errors,
)
from support.client_db import read_titles
from support.drift import assert_conn_state_stays
from support.fresh_world import count_letters, fresh_world_with_three_pages, wait_until_titles
from support.narrate import announce_result, say_both
from support.popup import assert_no_error_popups
from support.read_conn_state import wait_for_conn_state
from support.test_events import enter_lonewolf
from support.timeouts import secs

TITLE = "Grocery list"
EDITED_TITLE = "Grocery list v2"
GROCERY_BLOCK_COUNT = 6


def test_enter_lonewolf_is_stable_and_stays_local(two_browser_stage):
    stage = two_browser_stage
    brave, edge = stage.brave, stage.edge
    brave_errors = watch_js_errors(brave)
    edge_errors = watch_js_errors(edge)

    with announce_result(stage), report_js_errors_on_failure(brave_errors, edge_errors):
        say_both(stage, "Setup 1/1: fresh server, both browsers have the three sample pages")
        fresh_world_with_three_pages(stage)
        wait_for_conn_state(edge, "Connected")

        say_both(stage, "Step 1/3: Edge enters lonewolf on purpose")
        enter_lonewolf(edge)
        wait_for_conn_state(edge, "Lonewolf")
        assert_conn_state_stays(edge, "Lonewolf")

        say_both(stage, "Step 2/3: Edge enters lonewolf again, it should stay plain lonewolf")
        enter_lonewolf(edge)
        assert_conn_state_stays(edge, "Lonewolf")

        say_both(stage, "Step 3/3: Edge edits a title, the edit must stay on Edge")
        letters_before = count_letters()
        open_page_from_menu(edge, TITLE, GROCERY_BLOCK_COUNT)
        title_block = block_with_value(edge, TITLE)
        caret_to_end(title_block)
        title_block.press_sequentially(" v2")
        wait_until_titles(edge, [EDITED_TITLE])

        deadline = time.time() + secs()
        while time.time() < deadline:
            now = count_letters()
            assert now == letters_before, (
                f"the server received {now - letters_before} letter(s) from a lonewolf client"
            )
            time.sleep(0.1)

        brave_titles = read_titles(brave)
        assert EDITED_TITLE not in brave_titles, f"Brave saw Edge's lonewolf edit: {brave_titles}"
        assert TITLE in brave_titles, f"Brave lost {TITLE!r}: {brave_titles}"

        assert_no_error_popups(brave)
        assert_no_error_popups(edge)
    stage.close()
