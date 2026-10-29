"""Open Dev Tools, run every in-app check, assert none failed.

The checks themselves live in the app. This test verifies the app
agrees with itself. If a check fails, the failure detail is on the
dev page (expand the card) and in logcat under the DevCheck tag.
"""

from support import flows


def test_dev_checks(run):
    run.step(
        "Open Dev Tools",
        "Run all checks button visible",
        lambda: flows.open_dev_tools(run.driver),
    )

    def _run_and_assert():
        summary = flows.run_all_dev_checks(run.driver, run.serial, timeout=180)
        if summary["fail"] == 0:
            return
        detail = "\n".join(f"  - {cid}: {msg}" for cid, msg in summary["failures"])
        raise AssertionError(
            f"dev checks: {summary['fail']} failed, {summary['pass']} passed.\n{detail}"
        )

    run.step(
        "Run all checks",
        "DevCheck summary reports fail=0",
        _run_and_assert,
    )

    run.step(
        "Back to nav",
        "nav visible",
        lambda: flows.go_back_from_dev_tools(run.driver),
    )
