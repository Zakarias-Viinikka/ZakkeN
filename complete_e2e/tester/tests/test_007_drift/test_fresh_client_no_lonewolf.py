"""Tests-to-write #11: a client that has never logged in against this
server does not go lonewolf, and ends up holding the server's drift
version.

Both browsers wipe their own saved data (so neither has a stored drift
version) and register as new users. Both should reach Connected, never
a lonewolf state, and both should store the same drift version, since
they talk to the same server.

Not checked: that the stored value equals the server's own row. That
needs the server's drift table name and columns."""

import os
import re

from support.blocks import report_js_errors_on_failure, watch_js_errors
from support.drift import LONEWOLF, wait_for_stored_drift_version
from support.narrate import announce_result, say_both
from support.popup import assert_no_error_popups, clear_popup_log
from support.read_conn_state import read_conn_state, wait_for_conn_state
from support.setup_page import bring_up_registered
from support.test_events import reset_everything
from support.timeouts import ms


def test_fresh_client_does_not_go_lonewolf(two_browser_stage):
    stage = two_browser_stage
    brave, edge = stage.brave, stage.edge
    brave_errors = watch_js_errors(brave)
    edge_errors = watch_js_errors(edge)
    port = int(os.environ["SERVER_PORT"])

    with announce_result(stage), report_js_errors_on_failure(brave_errors, edge_errors):
        say_both(stage, "Setup 1/2: both browsers wipe their saved data and sign in as brand new users")
        for page, prefix in ((brave, "e2e_brave"), (edge, "e2e_edge")):
            reset_everything(page)
            page.wait_for_url(re.compile(r"/setup$"), timeout=ms())
            bring_up_registered(page, port, prefix)
            clear_popup_log(page)

        say_both(stage, "Step 1/2: neither browser should be lonewolf, both should be connected")
        for name, page in (("Brave", brave), ("Edge", edge)):
            wait_for_conn_state(page, "Connected")
            state = read_conn_state(page)
            assert state not in LONEWOLF, f"{name} went lonewolf on a fresh login: {state}"

        say_both(stage, "Step 2/2: both should have stored the server's drift version")
        brave_version = wait_for_stored_drift_version(brave)
        edge_version = wait_for_stored_drift_version(edge)
        assert brave_version == edge_version, (
            f"Brave stored {brave_version!r}, Edge stored {edge_version!r}, "
            "but both logged in to the same server"
        )

        assert_no_error_popups(brave)
        assert_no_error_popups(edge)
    stage.close()
