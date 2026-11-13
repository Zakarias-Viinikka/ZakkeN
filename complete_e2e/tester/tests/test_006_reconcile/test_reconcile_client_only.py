"""Reconcile with client-only pages.

Brave and Edge both have the three seed pages. Brave wipes the server.
Both go lonewolf. Brave resets + re-registers fresh (server still empty).
Edge opens /reconcile — all three of its pages are client-only (no server
copy). Submit opens the review; Send and finish sends CreateNewPage
letters for all three, wipes Edge locally, and pulls the fresh dataset
back. At the end both clients have the three titles and the server has
three pages.

The mismatched-page path (both sides have the same page id with different
content, requiring the diff viewer) is a separate test."""

import os
import re

from support.client_db import read_titles
from support.db import query, wait_until_server_has_pages
from support.fresh_world import SEED_TITLES, fresh_world_with_three_pages, wait_until_titles
from support.narrate import announce_result, say_both
from support.popup import assert_no_error_popups, clear_popup_log, popup
from support.read_conn_state import read_conn_state, wait_for_conn_state
from support.reconcile import (
    click_send_and_finish,
    click_submit,
    open_reconcile,
    wait_for_reconcile_to_finish,
)
from support.setup_page import bring_up_registered
from support.test_events import reset_everything, wipe_server


def test_reconcile_client_only(two_browser_stage):
    with announce_result(two_browser_stage):
        brave = two_browser_stage.brave
        edge = two_browser_stage.edge
        port = int(os.environ["SERVER_PORT"])

        say_both(two_browser_stage, "Setup 1/6: seed both browsers with three pages")
        fresh_world_with_three_pages(two_browser_stage)

        say_both(two_browser_stage, "Setup 2/6: brave wipes the server")
        wipe_server(brave)

        say_both(
            two_browser_stage,
            "Setup 3/6: edge goes lonewolf (brave is the wiper, so brave stays connected)",
        )
        try:
            wait_for_conn_state(
                edge, ("Lonewolf", "LonewolfAndOffline")
            )
        except AssertionError as e:
            state = read_conn_state(edge)
            url = edge.evaluate("window.location.href")
            popup(edge, f"edge stuck at state={state} url={url}")
            raise AssertionError(f"{e} (url={url})")
        clear_popup_log(brave)
        clear_popup_log(edge)

        say_both(two_browser_stage, "Setup 4/6: brave resets and re-registers fresh")
        reset_everything(brave)
        from support.timeouts import ms
        brave.wait_for_url(re.compile(r"/setup$"), timeout=ms())
        bring_up_registered(brave, port, "e2e_brave")
        clear_popup_log(brave)

        say_both(two_browser_stage, "Setup 5/6: edge opens reconcile")
        open_reconcile(edge)

        say_both(two_browser_stage, "Setup 6/6: edge clicks Submit, then Send and finish")
        click_submit(edge)
        click_send_and_finish(edge)
        wait_for_reconcile_to_finish(edge)

        # The reload leaves edge on /reconcile; navigate back to /Menu to
        # see the titles.
        edge.evaluate(
            "window.dispatchEvent(new CustomEvent('navigate', { detail: '/Menu' }))"
        )

        wait_until_titles(edge, SEED_TITLES)
        wait_until_server_has_pages(3)

        print("EDGE titles:", read_titles(edge))
        print("server pages:", query("SELECT COUNT(*) FROM pages")[0][0])

        assert_no_error_popups(brave)
        assert_no_error_popups(edge)

        popup(edge, "TEST DONE")