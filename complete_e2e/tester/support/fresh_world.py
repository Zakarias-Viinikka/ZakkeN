"""A known clean world for tests that need one: server wiped, both
browsers reset and re-registered, the three seed pages inserted by Brave
and received by Edge. Plus small server-side waits that watch the db
instead of sleeping."""

import os
import re
import time

from support.client_db import read_titles
from support.db import query, wait_until_server_has_pages
from support.narrate import app_said, say, say_both
from support.popup import clear_popup_log
from support.seconds import seconds
from support.setup_page import bring_up_registered
from support.test_events import insert_three_pages, reset_everything, wipe_server

SEED_TITLES = ["Grocery list", "Work notes", "Ideas"]
LETTERS_TABLE = "love_letters_from_clients"


def fresh_world_with_three_pages(stage) -> None:
    port = int(os.environ["SERVER_PORT"])
    brave, edge = stage.brave, stage.edge

    say_both(stage, "Setup 1/4: clearing everything stored on the server")
    users_before = count_users()
    wipe_server(brave)
    wait_until_users_drop_below(users_before)

    for page, prefix, label in ((brave, "e2e_brave", "brave"), (edge, "e2e_edge", "edge")):
        say_both(stage, f"Setup 2/4: wiping {label.capitalize()}'s saved data and signing it in as a new user")
        reset_everything(page)
        page.wait_for_url(re.compile(r"/setup$"), timeout=seconds(5))
        bring_up_registered(page, port, prefix)
        clear_popup_log(page)
        say(page, f"Setup 2/4: {label.capitalize()} is signed in as a new user")

    say_both(stage, "Setup 3/4: Brave creates three sample pages, Edge should receive them")
    insert_three_pages(brave)
    wait_until_server_has_pages(3)
    wait_until_titles(edge, SEED_TITLES)
    wait_until_letters_settle()
    clear_popup_log(brave)
    clear_popup_log(edge)
    say_both(stage, "Setup 4/4: ready, both browsers have the three sample pages")


def count_users() -> int:
    return query("SELECT COUNT(*) FROM registered_users")[0][0]


def wait_until_users_drop_below(before: int, timeout_secs: float = 5.0) -> None:
    deadline = time.time() + timeout_secs
    while time.time() < deadline:
        if count_users() < before:
            return
        time.sleep(0.1)
    raise AssertionError(
        f"registered_users still has {count_users()} rows after wipe_server (had {before})"
    )


def wait_until_titles(page, wanted: list[str], timeout_secs: float = 8.0) -> None:
    deadline = time.time() + timeout_secs
    titles: list[str] = []
    while time.time() < deadline:
        titles = read_titles(page)
        if all(t in titles for t in wanted):
            return
        time.sleep(0.2)
    raise AssertionError(f"titles never reached {wanted}, last saw {titles}")


def count_letters() -> int:
    return query(f"SELECT COUNT(*) FROM {LETTERS_TABLE}")[0][0]


def wait_for_new_letter(before: int, page=None, timeout_secs: float = 5.0) -> int:
    """Waits until the server holds more letters than `before`. Returns the
    new count, so the next edit can pass it as its own `before`."""
    deadline = time.time() + timeout_secs
    while time.time() < deadline:
        now = count_letters()
        if now > before:
            return now
        time.sleep(0.1)
    raise AssertionError(
        f"the server never received the edit (waited {timeout_secs:.0f}s).{app_said(page)}"
    )


def wait_until_letters_settle(quiet_secs: float = 0.6, timeout_secs: float = 10.0) -> None:
    """Seeding sends a burst of letters. Wait until the count stops moving
    so a test's own `before` count isn't polluted by the tail of the seed."""
    deadline = time.time() + timeout_secs
    last = count_letters()
    last_change = time.time()
    while time.time() < deadline:
        time.sleep(0.1)
        now = count_letters()
        if now != last:
            last = now
            last_change = time.time()
        elif time.time() - last_change >= quiet_secs:
            return
    raise AssertionError(f"letters at the server kept growing for {timeout_secs}s after seeding")