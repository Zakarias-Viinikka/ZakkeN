def seconds(n: float) -> int:
    """Seconds to milliseconds, for Playwright's timeout= arguments.
    Not multiplied by the speed setting — timeouts are failure detection,
    not pacing."""
    return int(n * 1000)