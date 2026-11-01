def seconds(n: float) -> int:
    """Seconds to milliseconds, for Playwright's timeout= arguments."""
    return int(n * 1000)
