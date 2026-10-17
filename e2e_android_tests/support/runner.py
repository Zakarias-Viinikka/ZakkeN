import json
import re
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

    def mark_session_failed(self, message):
        self.session_failed = True
        self.session_error = message

    def step(self, description, expected, action):
        self.step_index += 1
        idx = self.step_index
        slug = re.sub(r"[^a-z0-9]+", "_", description.lower()).strip("_")[:40]
        screenshot = self.run_dir / "screenshots" / f"step_{idx:03d}_{slug}.png"

        entry = {
            "index": idx,
            "description": description,
            "expected": expected,
            "observed": "",
            "screenshot": "",
            "status": "pending",
        }

        try:
            action()
            entry["status"] = "passed"
        except Exception as exc:
            entry["status"] = "failed"
            entry["observed"] = f"{type(exc).__name__}: {exc}"
            entry["screenshot"] = self._snap(screenshot)
            try:
                source = self.driver.page_source
                source_path = screenshot.with_suffix(".xml")
                source_path.write_text(source)
                entry["observed"] += f"  [screen XML: {source_path.name}]"
            except Exception:
                pass
            self.steps.append(entry)
            raise

        entry["screenshot"] = self._snap(screenshot)
        self.steps.append(entry)

    def _snap(self, path):
        try:
            self.driver.save_screenshot(str(path))
            return str(path)
        except Exception:
            return ""

    def finalize(self):
        if self._finalized:
            return
        self._finalized = True

        if self.logcat_text:
            (self.run_dir / "logcat.txt").write_text(self.logcat_text)

        report = {
            "run_dir": str(self.run_dir),
            "device_serial": self.serial,
            "started_at": self.started_at,
            "finished_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "session_failed": self.session_failed,
            "session_error": self.session_error,
            "steps": self.steps,
        }
        (self.run_dir / "report.json").write_text(json.dumps(report, indent=2))
        (self.run_dir / "report.md").write_text(self._render_md(report))

    def _render_md(self, report):
        lines = [
            "# UI test run",
            "",
            f"- Device: `{report['device_serial'] or '(unset)'}`",
            f"- Started: {report['started_at']}",
            f"- Finished: {report['finished_at']}",
        ]

        if report["session_failed"]:
            lines.append("- **Session failed before any test ran.**")
            lines.append("")
            lines.append("```")
            lines.append(report["session_error"])
            lines.append("```")
            return "\n".join(lines) + "\n"

        failures = [s for s in report["steps"] if s["status"] == "failed"]
        lines.append(f"- Result: **{'FAIL' if failures else 'PASS'}**")
        lines.append("")
        lines.append("## Steps")
        lines.append("")

        for s in report["steps"]:
            lines.append(f"### {s['index']}. {s['description']} — {s['status'].upper()}")
            lines.append("")
            lines.append(f"Expected: {s['expected']}")
            if s["status"] == "failed":
                lines.append("")
                lines.append(f"Observed: {s['observed']}")
            if s.get("screenshot"):
                rel = Path(s["screenshot"]).relative_to(self.run_dir)
                lines.append("")
                lines.append(f"Screenshot: `{rel}`")
            lines.append("")

        return "\n".join(lines) + "\n"
