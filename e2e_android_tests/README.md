# e2e_android_tests

AI-authored end-to-end UI regression tests for the z.zndroid Android app.
Driven by Appium UiAutomator2 over adb against a physical phone.

## One-time prerequisites

Android platform-tools on PATH, so `adb` works.

Appium 2.x installed globally:

    npm i -g appium
    appium driver install uiautomator2

Python 3.10+ with a virtualenv:

    cd ~/ProgStuff/ZakkeN/e2e_android_tests
    python3 -m venv .venv
    .venv/bin/pip install -r requirements.txt

Make the run script executable once:

    chmod +x ~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh

## Running

Connect the physical phone, unlock it, keep it connected during the run.
The app must already be installed; building and installing from Android
Studio is fine. Then run:

    ./run-ui-tests.sh

The script starts Appium if it's not already running, checks the device and
app, runs the suite, writes reports, and prints where they are.

## Reading results

Each run writes to `artifacts/<timestamp>/`:

report.md is a human-readable summary with pass/fail per step. report.json
is the same information, structured. logcat.txt is the Android log buffer
captured at the end of the run. artifacts/latest points at the most
recent run.

A behavioral step failing is what fails the run.

## Adding a new test

Look at `tests/test_page_persistence.py` for the shape. Reuse `support/flows.py`
helpers (create, open, add a block, rename, delete) and `support/ui.py`
primitives (find by text, click by text, wait for text). Use the reserved
prefix `__AI_AUTOTEST__` for any page you create so the next run's cleanup
can find and remove it. Wrap each user action in
`run.step(description, expected, callable)`. Do not modify production app
code.
