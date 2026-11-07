"""Audit findings #1 and #3: text edits that must survive a reload.
Both are expected to FAIL today. A reload reads the client's yrs doc, so
the check sees what yrs holds, not what the textarea showed."""

from support.blocks import (
    block_with_value,
    caret_to_end,
    open_page_from_menu,
    reload_and_wait_for_blocks,
    report_js_errors_on_failure,
    select_range,
    wait_for_value_in_blocks,
    watch_js_errors,
)
from support.fresh_world import (
    count_letters,
    fresh_world_with_three_pages,
    wait_for_new_letter,
)
from support.narrate import announce_result, say_both


def test_replacing_a_selection_with_shorter_text_keeps_the_typed_text(two_browser_stage):
    """Select 'il' in 'Milk', type 'X'. Expect 'MXk' after reload.
    get_diff only emits a Delete here, so the X is never sent."""
    stage = two_browser_stage
    brave = stage.brave
    errors = watch_js_errors(brave)

    with announce_result(stage), report_js_errors_on_failure(errors):
        fresh_world_with_three_pages(stage)

        say_both(stage, "1/5 Brave opens the Grocery list page")
        open_page_from_menu(brave, "Grocery list", 6)

        say_both(stage, "2/5 Brave selects 'il' inside 'Milk' and types X over it")
        before = count_letters()
        select_range(block_with_value(brave, "Milk"), 1, 3)
        brave.keyboard.type("X")

        say_both(stage, "3/5 waiting for the server to receive the edit", hold=0.3)
        wait_for_new_letter(before, brave)

        say_both(stage, "4/5 server has it, reloading Brave to see what was really saved")
        reload_and_wait_for_blocks(brave, 6)
        values = wait_for_value_in_blocks(brave, "MXk")

        say_both(stage, f"5/5 we expected 'MXk', after the reload Brave shows: {values}", hold=2.0)
        assert "MXk" in values, (
            f"the typed X was lost: after the reload we expected 'MXk' but the page shows {values}"
        )

    stage.close()


def test_edit_after_swedish_characters_lands_in_the_right_place(two_browser_stage):
    """Type 'åäö' at the end of 'Milk', let it sync, then type 'X' at the
    end. Expect 'MilkåäöX' after reload. The two edits are synced separately
    on purpose: batched together they would combine into one insert and hide
    the char/byte offset mismatch."""
    stage = two_browser_stage
    brave = stage.brave
    errors = watch_js_errors(brave)

    with announce_result(stage), report_js_errors_on_failure(errors):
        fresh_world_with_three_pages(stage)

        say_both(stage, "1/6 Brave opens the Grocery list page")
        open_page_from_menu(brave, "Grocery list", 6)

        say_both(stage, "2/6 Brave types åäö at the end of 'Milk'")
        before = count_letters()
        caret_to_end(block_with_value(brave, "Milk"))
        brave.keyboard.insert_text("åäö")
        before = wait_for_new_letter(before, brave)

        say_both(stage, "3/6 the server has åäö, now Brave types an X after it")
        caret_to_end(block_with_value(brave, "Milkåäö"))
        brave.keyboard.type("X")

        say_both(stage, "4/6 waiting for the server to receive the X", hold=0.3)
        wait_for_new_letter(before, brave)

        say_both(stage, "5/6 server has both edits, reloading Brave to see what was really saved")
        reload_and_wait_for_blocks(brave, 6)
        values = wait_for_value_in_blocks(brave, "MilkåäöX")

        say_both(stage, f"6/6 we expected 'MilkåäöX', after the reload Brave shows: {values}", hold=2.0)
        assert "MilkåäöX" in values, (
            f"the X ended up in the wrong place: we expected 'MilkåäöX' but the page shows {values}"
        )

    stage.close()