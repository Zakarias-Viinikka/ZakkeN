import os
import re

from playwright.sync_api import expect

from support.client_db import read_client_state, read_titles
from support.db import wait_until_server_has_pages
from support.popup import assert_no_error_popups, clear_popup_log, popup
from support.seconds import seconds
from support.setup_page import bring_up_registered
from support.test_events import insert_three_pages, reset_everything, wipe_server


def both(stage, message):
    popup(stage.brave, message)
    popup(stage.edge, message)


HARDCODED_TITLE = "Grocery list"
HARDCODED_TITLE_BLOCK_COUNT = 6


def test_two_clients_after_wipe(two_browser_stage, pause):
    brave = two_browser_stage.brave
    edge = two_browser_stage.edge
    port = int(os.environ["SERVER_PORT"])

    both(two_browser_stage, "0/7 both registered - test starting")
    pause(1)

    both(two_browser_stage, "1/7 brave wipes the server")
    wipe_server(brave)
    pause(2)

    both(two_browser_stage, "2/7 brave resets")
    reset_everything(brave)
    from support.timeouts import ms
    brave.wait_for_url(re.compile(r"/setup$"), timeout=ms())
    bring_up_registered(brave, port, "e2e_brave")
    clear_popup_log(brave)
    popup(brave, "3/7 brave is fresh and registered again")
    pause(1)

    both(two_browser_stage, "4/7 edge resets")
    reset_everything(edge)
    from support.timeouts import ms
    edge.wait_for_url(re.compile(r"/setup$"), timeout=ms())
    bring_up_registered(edge, port, "e2e_edge")
    clear_popup_log(edge)
    popup(edge, "5/7 edge is fresh and registered again")
    pause(1)

    both(two_browser_stage, "6/7 brave inserts 3 pages - edge should receive them")
    insert_three_pages(brave)
    wait_until_server_has_pages(3)
    pause(2)

    titles = read_titles(edge)
    print("EDGE titles:", titles)
    print("EDGE state:", read_client_state(edge))
    print("BRAVE state:", read_client_state(brave))
    assert len(titles) == 3, f"Edge has {len(titles)} titles, wanted 3: {titles}"
    for expected in ["Grocery list", "Work notes", "Ideas"]:
        assert expected in titles, f"Edge titles missing {expected!r}: {titles}"

    assert_no_error_popups(brave)
    assert_no_error_popups(edge)

    # 7/7 - open the hardcoded page visually and count blocks.
    both(two_browser_stage, f"7/7 opening {HARDCODED_TITLE} on edge")
    pause(1)

    target = edge.locator("button.title_button", has_text=HARDCODED_TITLE)
    from support.timeouts import ms
    expect(target).to_have_count(1, timeout=ms())
    target.click()
    from support.timeouts import ms
    expect(edge).to_have_url(re.compile(r"/MainContent/"), timeout=ms())
    from support.timeouts import ms
    edge.wait_for_selector("textarea.ps-content", timeout=ms())
    pause(2)

    popup(edge, f"counting blocks in {HARDCODED_TITLE}")
    pause(1)
    block_count = edge.locator("textarea.ps-content").count()
    print(f"EDGE {HARDCODED_TITLE} block count: {block_count}")
    popup(edge, f"{HARDCODED_TITLE} has {block_count} blocks")
    assert block_count == HARDCODED_TITLE_BLOCK_COUNT, (
        f"{HARDCODED_TITLE} expected {HARDCODED_TITLE_BLOCK_COUNT} blocks, got {block_count}"
    )

    both(two_browser_stage, "TEST DONE")