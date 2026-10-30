import json
import re
import subprocess
import time
from pathlib import Path


class UiTestRun:
    def __init__(self, run_dir, serial):
        self.run_dir = Path(run_dir)
        self.serial = serial
        self.driver = None
        self.steps = []
        self.step_index = 0
        self.started_at = time.strftime("%Y-%m-%dT%H:%M:%S")
        self.session_failed = False
        self.session_error = ""
        self.logcat_text = ""
        self._finalized = False
        self._write()

    def mark_session_failed(self, message):
        self.session_failed = True
        self.session_error = message
        self._write()

    def _logcat_clear(self):
        try:
            subprocess.run(
                ["adb", "-s", self.serial, "logcat", "-c"],
                capture_output=True, timeout=5,
            )
        except Exception:
            pass

    def _logcat_read(self):
        """Full dump. Only used on failure."""
        try:
            r = subprocess.run(
                ["adb", "-s", self.serial, "logcat", "-d"],
                capture_output=True, text=True, timeout=15,
            )
            return (r.stdout or "").splitlines()
        except Exception as e:
            return [f"(logcat read failed: {e})"]

    def _logcat_read_quiet(self):
        """Filtered read. Logcat applies the tag filter on-device, so only
        matching lines come back over adb — the whole buffer isn't shipped."""
        try:
            r = subprocess.run(
                ["adb", "-s", self.serial, "logcat", "-d",
                 "-s", "GlobalPopup:V", "DevCheck:V", "EditBlockExec:I",
                 "OpsCreateAndRename:I", "RenamePage:I", "AndroidRuntime:E"],
                capture_output=True, text=True, timeout=10,
            )
            return (r.stdout or "").splitlines()
        except Exception as e:
            return [f"(logcat read failed: {e})"]

    def _interesting(self, raw):
        """Keep lines mentioning GlobalPopup, FATAL, or errors from the app."""
        out = []
        for line in raw:
            if "GlobalPopup" in line:
                out.append(line)
            elif "DevCheck" in line:
                out.append(line)
            elif "EditBlockExec" in line:
                out.append(line)
            elif "OpsCreateAndRename" in line:
                out.append(line)
            elif "RenamePage" in line:
                out.append(line)
            elif "FATAL" in line:
                out.append(line)
            elif "z.zndroid" in line and (" E " in line or " E/" in line):
                out.append(line)
            elif line.startswith("E/"):
                out.append(line)
        return out[-40:]

    def _ensure_nav(self):
        """Between tests, the app may be anywhere. If it isn't on nav,
        press back until it is. Called once at the start of every test via
        `before_test()`. Idempotent — no-op if already on nav."""
        from . import flows, ui
        for _ in range(3):
            try:
                if ui.text_present(self.driver, flows.NEW_PAGE_BUTTON):
                    return
            except Exception:
                pass
            try:
                ui.press_back(self.driver)
            except Exception:
                return
            time.sleep(0.4)
        try:
            flows.ensure_nav_page(self.driver, timeout=5)
        except Exception:
            pass

    def before_test(self):
        self._ensure_nav()

    def step(self, description, expected, action):
        from . import ui

        self.step_index += 1
        idx = self.step_index

        self._logcat_clear()
        pre = ui.get_popup(self.driver)
        entry = {
            "index": idx,
            "description": description,
            "expected": expected,
            "observed": "",
            "popup_before": pre[0] if pre else "",
            "popup_after": "",
            "logcat_interesting": [],
            "logcat_tail": [],
            "status": "running",
        }
        self.steps.append(entry)
        self._write()

        def _collect_full():
            time.sleep(0.3)
            post = self._popup_safe()
            if post:
                entry["popup_after"] = post[0]
            raw = self._logcat_read()
            entry["logcat_interesting"] = self._interesting(raw)
            entry["logcat_tail"] = raw[-30:] if raw else []

        def _collect_quiet():
            time.sleep(0.3)
            post = self._popup_safe()
            if post:
                entry["popup_after"] = post[0]
            entry["logcat_interesting"] = self._interesting(self._logcat_read_quiet())

        try:
            action()
            entry["status"] = "passed"
        except Exception as exc:
            entry["status"] = "failed"
            entry["observed"] = f"{type(exc).__name__}: {exc}"
            _collect_full()
            current = self._current_package()
            if current and current != "z.zndroid":
                entry["observed"] += (
                    f"  |  APP NO LONGER FOREGROUND (current package: {current})"
                )
                if not entry["logcat_interesting"]:
                    entry["observed"] += (
                        "  |  no crash or GlobalPopup in logcat — the app died silently"
                    )
            if entry["logcat_interesting"]:
                entry["observed"] += (
                    "  |  logcat: " + " ;; ".join(entry["logcat_interesting"])
                )
            self._write()
            raise

        _collect_quiet()
        errored = [
            l for l in entry["logcat_interesting"]
            if "GlobalPopup" in l or "FATAL" in l
        ]
        if errored:
            entry["status"] = "failed"
            entry["observed"] = "app logged during step: " + " ;; ".join(errored)
            _collect_full()
            self._write()
            raise AssertionError(entry["observed"])

        self._write()

    def restart_app_and_wait(self, settle=2.0):
        from .driver import APP_PACKAGE
        try:
            self.driver.terminate_app(APP_PACKAGE)
        except Exception:
            pass
        time.sleep(0.5)
        self.driver.activate_app(APP_PACKAGE)
        time.sleep(settle)

    def assert_sync(self, page_id, timeout=10):
        import subprocess as _sp
        try:
            _sp.run(["adb", "-s", self.serial, "logcat", "-c"],
                    capture_output=True, timeout=5)
        except Exception:
            pass
        try:
            _sp.run(
                ["adb", "-s", self.serial, "shell", "am", "broadcast",
                 "-a", "z.zndroid.debug.CHECK_SYNC", "-p", "z.zndroid",
                 "--es", "pageId", page_id],
                capture_output=True, timeout=10,
            )
        except Exception as e:
            raise AssertionError(f"could not broadcast CHECK_SYNC: {e}")
        marker = f"SyncCheck page={page_id}"
        deadline = time.time() + timeout
        while time.time() < deadline:
            try:
                r = _sp.run(
                    ["adb", "-s", self.serial, "logcat", "-d", "-s",
                     "TestTransactionReceiver:V"],
                    capture_output=True, text=True, timeout=5,
                )
            except Exception:
                time.sleep(0.5); continue
            out = r.stdout or ""
            for line in out.splitlines():
                if marker not in line:
                    continue
                if " error=" in line:
                    raise AssertionError(f"sync check errored: {line.strip()}")
                if "synced=true" in line:
                    return
                if "synced=false" in line:
                    mismatches = [
                        l for l in out.splitlines()
                        if "SyncCheckMismatch" in l and f"page={page_id}" in l
                    ]
                    detail = " ;; ".join(m.strip() for m in mismatches[:5]) or "(no mismatch detail)"
                    raise AssertionError(
                        f"page {page_id!r} out of sync between yrs and every_block_in_existence. "
                        f"{line.strip()}  |  {detail}"
                    )
            time.sleep(0.4)
        raise AssertionError(
            f"sync check for page {page_id!r}: no SyncCheck log within {timeout}s"
        )

    def _popup_safe(self):
        from . import ui
        try:
            return ui.get_popup(self.driver)
        except Exception:
            return None

    def _current_package(self):
        try:
            return self.driver.current_package
        except Exception:
            return None

    def finalize(self):
        if self._finalized:
            return
        self._finalized = True
        try:
            raw = self._logcat_read()
            (self.run_dir / "logcat.txt").write_text("\n".join(raw))
        except Exception:
            pass
        self._write()

    def _write(self):
        report = {
            "run_dir": str(self.run_dir),
            "device_serial": self.serial,
            "started_at": self.started_at,
            "finished_at": (
                time.strftime("%Y-%m-%dT%H:%M:%S") if self._finalized else ""
            ),
            "in_progress": not self._finalized,
            "session_failed": self.session_failed,
            "session_error": self.session_error,
            "steps": self.steps,
        }
        try:
            (self.run_dir / "report.json").write_text(json.dumps(report, indent=2))
            (self.run_dir / "report.md").write_text(self._render_md(report))
        except Exception:
            pass

    def _render_md(self, report):
        lines = [
            "# UI test run",
            "",
            f"- Device: `{report['device_serial'] or '(unset)'}`",
            f"- Started: {report['started_at']}",
        ]
        if report["finished_at"]:
            lines.append(f"- Finished: {report['finished_at']}")
        else:
            lines.append("- **In progress** (rewritten after every step)")

        if report["session_failed"]:
            lines.append("")
            lines.append("- **Session failed before any test ran.**")
            lines.append("")
            lines.append("```")
            lines.append(report["session_error"])
            lines.append("```")
            return "\n".join(lines) + "\n"

        steps = report["steps"]
        failures = [s for s in steps if s["status"] == "failed"]
        running = [s for s in steps if s["status"] == "running"]

        if running or not steps:
            lines.append("- Result so far: **RUNNING**")
        else:
            lines.append(f"- Result: **{'FAIL' if failures else 'PASS'}**")

        lines.append("")
        lines.append("## Steps")
        lines.append("")

        for s in steps:
            lines.append(f"### {s['index']}. {s['description']} — {s['status'].upper()}")
            lines.append("")
            lines.append(f"Expected: {s['expected']}")
            if s["status"] == "failed":
                lines.append("")
                lines.append(f"Observed: {s['observed']}")
            if s.get("popup_before"):
                lines.append("")
                lines.append(f"Popup before step: {s['popup_before']!r}")
            if s.get("popup_after"):
                lines.append("")
                lines.append(f"Popup after step:  {s['popup_after']!r}")

            if s.get("logcat_interesting"):
                lines.append("")
                lines.append("Interesting logcat lines during this step:")
                lines.append("```")
                for l in s["logcat_interesting"]:
                    lines.append(l)
                lines.append("```")

            if s.get("logcat_tail"):
                lines.append("")
                lines.append("Raw logcat tail (last 30 lines during this step):")
                lines.append("```")
                for l in s["logcat_tail"]:
                    lines.append(l)
                lines.append("```")

            lines.append("")

        return "\n".join(lines) + "\n"