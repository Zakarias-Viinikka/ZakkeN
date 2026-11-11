"""One timeout number that every test uses. Change DEFAULT_SECS to
move every wait at once. If a specific call site genuinely needs more
than the default, that call site uses a documented constant right
beside itself with a one-line reason — do not silently pad."""

DEFAULT_SECS = 2.0


def ms() -> int:
    """Default timeout in milliseconds, for Playwright's timeout= args."""
    return int(DEFAULT_SECS * 1000)


def secs() -> float:
    """Default timeout in seconds, for python-side `timeout_secs=` args."""
    return DEFAULT_SECS
