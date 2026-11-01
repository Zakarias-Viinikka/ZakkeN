# What you need installed

Python 3, plus pytest, Playwright, and the Chromium browser Playwright uses.

## Linux MX

    python3 -m venv .venv
    source .venv/bin/activate
    pip install pytest playwright
    playwright install chromium

If `playwright install chromium` fails on missing system libraries:

    sudo apt-get install -y libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libgbm1 libasound2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libpango-1.0-0 libcairo2
    playwright install chromium

## Running

`c r` (your shell function; equivalently `./local_cargo.sh r` from `complete_e2e/`) does everything — builds, spawns, serves, runs pytest, tears down. `tester/run_tests.sh` runs pytest alone with the venv already on PATH.

Do NOT `source .venv/bin/activate` when running. `local_cargo.sh` and `run_tests.sh` prepend `.venv/bin` to PATH directly. Activating a copied venv points Python at the *original* venv (the activate script stores the old path), which causes headed tests to flake with `TargetClosedError` while headless passes. The activate step above is only for the one-time pip install.