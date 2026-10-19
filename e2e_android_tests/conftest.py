import os
import subprocess
import time
from pathlib import Path

import pytest

from support.driver import APP_PACKAGE, create_driver
from support.logcat import dump_logcat
from support.runner import UiTestRun
from support import flows, ui


def _broadcast(serial, action):
    subprocess.run(
        ["adb", "-s", serial, "shell", "am", "broadcast", "-a", action, "-p", APP_PACKAGE],
        check=False, capture_output=True, timeout=10,
    )


def _wait_tx_ok(serial, marker, timeout=20):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            result = subprocess.run(
                ["adb", "-s", serial, "logcat", "-d",
                 "-s", "TestTransactionReceiver:I", "-t", "100"],
                capture_output=True, text=True, timeout=5,
            )
        except Exception:
            time.sleep(0.5)
            continue
        for line in reversed((result.stdout or "").splitlines()):
            if marker in line:
                if "-> OK" in line:
                    return True
                if "-> FAIL" in line or "threw" in line:
                    return False
        time.sleep(0.5)
    return False


def _begin_tx(serial):
    subprocess.run(["adb", "-s", serial, "logcat", "-c"], check=False, capture_output=True)
    _broadcast(serial, "z.zndroid.debug.BEGIN_TEST_TX")
    if not _wait_tx_ok(serial, "BEGIN_TEST_TX"):
        raise RuntimeError(
            "BEGIN_TEST_TX did not report OK. Check adb logcat -s TestTransactionReceiver:I"
        )


def _end_tx_rollback(serial):
    _broadcast(serial, "z.zndroid.debug.END_TEST_TX_ROLLBACK")
    if not _wait_tx_ok(serial, "END_TEST_TX_ROLLBACK"):
        raise RuntimeError(
            "END_TEST_TX_ROLLBACK did not report OK. Transaction state is unknown."
        )


def _force_stop(serial):
    subprocess.run(
        ["adb", "-s", serial, "shell", "am", "force-stop", APP_PACKAGE],
        check=False, capture_output=True, timeout=5,
    )


def _reset_to_nav(driver, serial=None, timeout=20):
    # Force-stop via adb first. This kills the app process, which forces SQLite
    # to roll back any transaction left open by a crashed prior run. Without
    # this, a leftover transaction sits on the phone and every future write
    # appears to succeed but never commits.
    if serial:
        _force_stop(serial)
    else:
        try:
            driver.terminate_app(APP_PACKAGE)
        except Exception:
            pass
    driver.activate_app(APP_PACKAGE)
    ui.wait_for_text(driver, flows.NEW_PAGE_BUTTON, timeout=timeout)


@pytest.fixture(scope="session")
def run():
    run_dir = Path(os.environ.get("RUN_DIR", "./artifacts/_manual_run")).resolve()
    run_dir.mkdir(parents=True, exist_ok=True)
    (run_dir / "screenshots").mkdir(exist_ok=True)

    serial = os.environ.get("DEVICE_SERIAL", "")
    recorder = UiTestRun(run_dir=run_dir, serial=serial)

    try:
        recorder.driver = create_driver()
    except Exception as exc:
        recorder.mark_session_failed(str(exc))
        recorder.finalize()
        pytest.fail(f"Appium session failed ({type(exc).__name__}). See report.md for details.")

    tx_open = False
    setup_error = None
    try:
        try:
            _reset_to_nav(recorder.driver, serial=serial)
            # Sweep any legacy test pages from earlier non-transaction runs.
            # This is a real delete, before the transaction opens, so it persists.
            try:
                flows.cleanup_test_pages(recorder.driver, flows.PREFIX)
            except Exception:
                pass
            if serial:
                _begin_tx(serial)
                tx_open = True
        except Exception as exc:
            setup_error = exc
            recorder.mark_session_failed(f"setup failed: {exc}")
            pytest.fail(f"setup failed: {exc}")

        yield recorder
    finally:
        if tx_open:
            try:
                _end_tx_rollback(serial)
            except Exception as exc:
                recorder.steps.append({
                    "index": 999,
                    "description": "rollback external transaction",
                    "expected": "END_TEST_TX_ROLLBACK reports OK",
                    "observed": str(exc) + "  |  force-stopping app to release the transaction",
                    "screenshot": "",
                    "status": "failed",
                })
        any_failed = recorder.session_failed or any(
            s.get("status") == "failed" for s in recorder.steps
        )
        if any_failed:
            try:
                _force_stop(serial)
            except Exception:
                pass
            print("[e2e] run failed — app force-stopped; your phone is free", flush=True)
        else:
            try:
                _reset_to_nav(recorder.driver, serial=serial)
            except Exception:
                pass
        try:
            recorder.logcat_text = dump_logcat(serial) if serial else ""
        except Exception as exc:
            recorder.logcat_text = f"(logcat dump failed: {exc})"
        recorder.finalize()

@pytest.fixture(autouse=True)
def _reset_to_nav_before_test(run):
    run.before_test()
    yield
