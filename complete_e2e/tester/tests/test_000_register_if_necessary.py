import random
import re
import time

from playwright.sync_api import expect

from support import db
from support.popup import popup
from support.seconds import seconds
from support.setup_page import on_setup_page, setup_page_register


def test_register_if_necessary(web_app, pause):
    from support.timeouts import ms
    web_app.wait_for_url(re.compile(r"/(setup|Menu)$"), timeout=ms())

    if not on_setup_page(web_app):
        expect(web_app).to_have_url(re.compile(r"/Menu$"))
        popup(web_app, "already registered")
        pause(1)
        return

    name = f"e2e_{random.randint(100000, 999999)}"
    setup_page_register(web_app, name)
    pause(1)
    expect(web_app).to_have_url(re.compile(r"/Menu$"))

    from support import speed
    deadline = time.time() + 3
    while time.time() < deadline:
        rows = db.query("SELECT * FROM registered_users")
        if any(name in [str(v) for v in row] for row in rows):
            popup(web_app, f"registered as {name}")
            pause(2)
            return
        time.sleep(0.1)
    raise AssertionError(f"{name} not in server's registered_users after 3s")