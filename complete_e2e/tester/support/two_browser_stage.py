import tkinter as tk

from playwright.sync_api import sync_playwright


class TwoBrowserStage:
    """Two Chromium windows side by side, with a method per layout.

    The caller uses `.brave` and `.edge` as ordinary Playwright Pages.
    Everything about positioning and window ids stays inside this file.
    """

    def __init__(self, brave_browser, brave_cdp, brave_window_id,
                 edge_browser, edge_cdp, edge_window_id, screen_w, screen_h):
        self._brave_browser = brave_browser
        self._brave_cdp = brave_cdp
        self._brave_window_id = brave_window_id
        self._edge_browser = edge_browser
        self._edge_cdp = edge_cdp
        self._edge_window_id = edge_window_id
        self._screen_w = screen_w
        self._screen_h = screen_h
        self.brave = brave_browser.contexts[0].pages[0]
        self.edge = edge_browser.contexts[0].pages[0]

    @classmethod
    def launch(cls, playwright, url="about:blank"):
        screen_w, screen_h = _get_screen_size()

        brave_browser = playwright.chromium.launch(headless=False)
        brave_page = brave_browser.new_context(no_viewport=True).new_page()
        brave_page.goto(url)
        _add_label(brave_page, "BRAVE", "#1a3fd4")
        brave_cdp, brave_window_id = _get_window(brave_browser)

        edge_browser = playwright.chromium.launch(headless=False)
        edge_page = edge_browser.new_context(no_viewport=True).new_page()
        edge_page.goto(url)
        _add_label(edge_page, "EDGE", "#c81a1a")
        edge_cdp, edge_window_id = _get_window(edge_browser)

        stage = cls(
            brave_browser, brave_cdp, brave_window_id,
            edge_browser, edge_cdp, edge_window_id,
            screen_w, screen_h,
        )
        stage.share_screen_with_edge_and_brave()
        return stage

    def make_brave_big(self):
        _show_at(self._brave_cdp, self._brave_window_id, 0, 0, self._screen_w, self._screen_h)
        _minimize(self._edge_cdp, self._edge_window_id)

    def make_edge_big(self):
        _minimize(self._brave_cdp, self._brave_window_id)
        _show_at(self._edge_cdp, self._edge_window_id, 0, 0, self._screen_w, self._screen_h)

    def share_screen_with_edge_and_brave(self):
        half = self._screen_w // 2
        _show_at(self._brave_cdp, self._brave_window_id, 0, 0, half, self._screen_h)
        _show_at(self._edge_cdp, self._edge_window_id, half, 0, half, self._screen_h)

    def close(self):
        self._brave_browser.close()
        self._edge_browser.close()


def _get_screen_size():
    root = tk.Tk()
    root.withdraw()
    width = root.winfo_screenwidth()
    height = root.winfo_screenheight()
    root.destroy()
    return width, height


def _get_window(browser):
    cdp = browser.new_browser_cdp_session()
    targets = cdp.send("Target.getTargets")
    pages = [t for t in targets["targetInfos"] if t["type"] == "page"]
    if not pages:
        raise RuntimeError("browser has no page target yet")
    result = cdp.send("Browser.getWindowForTarget", {"targetId": pages[0]["targetId"]})
    return cdp, result["windowId"]


def _show_at(cdp, window_id, x, y, w, h):
    cdp.send(
        "Browser.setWindowBounds",
        {"windowId": window_id, "bounds": {"windowState": "normal"}},
    )
    cdp.send(
        "Browser.setWindowBounds",
        {"windowId": window_id, "bounds": {"left": x, "top": y, "width": w, "height": h}},
    )


def _minimize(cdp, window_id):
    cdp.send(
        "Browser.setWindowBounds",
        {"windowId": window_id, "bounds": {"windowState": "minimized"}},
    )


def _add_label(page, text, color):
    page.evaluate(
        f"""
        () => {{
            const d = document.createElement('div');
            d.id = '__spike_label__';
            d.textContent = '{text}';
            d.style.cssText = 'position:fixed;top:0;right:0;padding:12px 24px;background:{color};color:white;font-size:28px;font-weight:bold;z-index:2147483647;font-family:sans-serif;pointer-events:none;';
            document.body.appendChild(d);
        }}
        """
    )
