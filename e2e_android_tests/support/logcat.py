import subprocess


def dump_logcat(serial, lines=5000):
    try:
        result = subprocess.run(
            ["adb", "-s", serial, "logcat", "-d", "-v", "threadtime", "-t", str(lines)],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return result.stdout or "(empty logcat)"
    except Exception as exc:
        return f"(failed to dump logcat: {exc})"
