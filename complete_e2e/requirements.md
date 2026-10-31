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