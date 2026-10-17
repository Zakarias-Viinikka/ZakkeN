import os
from pathlib import Path

import pytest

from support.driver import create_driver
from support.logcat import dump_logcat
from support.runner import UiTestRun


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

    yield recorder

    try:
        recorder.logcat_text = dump_logcat(serial) if serial else ""
    except Exception as exc:
        recorder.logcat_text = f"(logcat dump failed: {exc})"
    recorder.finalize()
