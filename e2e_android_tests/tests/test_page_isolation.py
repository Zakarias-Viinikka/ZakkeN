"""Three pages each with their own body content: after process kill, each
page must show only its own text. Catches SQLite mirror cross-wiring.

If you are unsure whether the user expects this behaviour or the current
code's behaviour to be the source of truth, STOP and ask the human. Do
not silently pick one.
"""
import time

from support import flows, ui
from support.flows import PREFIX


def test_page_isolation(run):
    ts = time.strftime("%Y%m%d_%H%M%S")
    names = [f"{PREFIX}{ts}_a", f"{PREFIX}{ts}_b", f"{PREFIX}{ts}_c"]
    bodies = [f"alpha_{ts}", f"beta_{ts}", f"gamma_{ts}"]

    for i, name in enumerate(names):
        run.step(f"Create page {name}", f"{name} visible in nav",
                 lambda n=name: flows.create_page(run.driver, n))
        run.step(f"Open page {name}", "page view visible",
                 lambda n=name: flows.open_page(run.driver, n))
        run.step(f"Add unique body '{bodies[i]}'", "body visible",
                 lambda b=bodies[i]: flows.add_body_block(run.driver, b))
        run.step(f"Back to nav", "nav visible",
                 lambda: flows.go_back_to_nav(run.driver))

    run.step("Kill and relaunch app", "app back on nav",
             lambda: _kill_and_relaunch(run))

    for i, name in enumerate(names):
        run.step(f"Open page {name}", "page view visible",
                 lambda n=name: flows.open_page(run.driver, n))
        run.step(f"Page {name} shows its body '{bodies[i]}'",
                 f"{bodies[i]!r} present",
                 lambda b=bodies[i]: ui.wait_for_text(run.driver, b, timeout=10))
        for j, other in enumerate(bodies):
            if j == i:
                continue
            run.step(f"Page {name} does NOT show '{other}'",
                     f"{other!r} absent",
                     lambda o=other: _assert_absent(run.driver, o))
        run.step(f"Back to nav from {name}", "nav visible",
                 lambda: flows.go_back_to_nav(run.driver))


def _assert_absent(driver, text):
    if ui.text_present(driver, text):
        raise AssertionError(f"page shows foreign text {text!r}")


def _kill_and_relaunch(run):
    run.restart_app_and_wait(settle=3.0)
    flows.ensure_nav_page(run.driver, timeout=15)
