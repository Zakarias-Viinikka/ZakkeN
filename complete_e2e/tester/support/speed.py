"""Per-test speed multiplier. Every hardcoded wait/timeout in the tester
goes through this module so one json file controls overall pace.

Multiplier > 1 means slower (all waits stretched).
Multiplier < 1 means faster (all waits shrunk).
"""

import json
import os
import time

_CONFIG_PATH = os.path.join(
    os.path.dirname(os.path.dirname(__file__)), "speed_config.json"
)
_DEFAULT = 1.0
_current = _DEFAULT


def _load_config() -> dict:
    try:
        with open(_CONFIG_PATH) as f:
            return json.load(f)
    except Exception:
        return {}


def multiplier_for(module_name: str) -> float:
    cfg = _load_config()
    if module_name in cfg:
        return float(cfg[module_name])
    short = module_name.rsplit(".", 1)[-1]
    if short in cfg:
        return float(cfg[short])
    return _DEFAULT


def set_current(multiplier: float) -> None:
    global _current
    _current = float(multiplier)


def current() -> float:
    return _current


def seconds(n: float) -> int:
    """Milliseconds for playwright timeouts, multiplied."""
    return int(n * 1000 * _current)


def sleep(n: float) -> None:
    """A settling wait, multiplied."""
    time.sleep(n * _current)


def budget(n: float) -> float:
    """Seconds for a total timeout budget in a polling loop, multiplied."""
    return n * _current
