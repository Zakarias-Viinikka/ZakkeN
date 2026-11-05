# New pytester spike guide

A spike is a tiny throwaway setup for testing Python-tester mechanics
without dragging the real app into it. Two browser windows, arbitrary
junk, done. The proven shape moves into `complete_e2e/` afterwards.

## Folder shape

One folder per spike, under `py_lab/`. Inside it:

    spike_name/
      orchestrator/
        Cargo.toml
        src/
          main.rs
          main_for_running_the_python_spike.rs
      local_cargo.sh
      spike.py
      requirements.txt

The orchestrator is deliberately tiny. Its only job is `c r` runs the
python spike. If a spike needs to serve something or start something
before python runs, extend the orchestrator then — not before.

## orchestrator/Cargo.toml

    [package]
    name = "orchestrator"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    tokio = { version = "1", features = ["rt-multi-thread", "macros", "process"] }

    [workspace]

The `[workspace]` at the end stops Cargo from walking up and finding a
different workspace.

## orchestrator/src/main.rs

    mod main_for_running_the_python_spike;

    #[tokio::main]
    async fn main() {
        main_for_running_the_python_spike::main_for_running_the_python_spike().await;
    }

## orchestrator/src/main_for_running_the_python_spike.rs

Runs `python3 spike.py` in the parent folder, forwards every argument,
exits with python's exit code.

    use std::path::PathBuf;

    pub async fn main_for_running_the_python_spike() {
        let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut cmd = tokio::process::Command::new("python3");
        cmd.arg("spike.py");
        for a in &args {
            cmd.arg(a);
        }
        cmd.current_dir(&folder);

        let status = cmd.status().await.expect("failed to run python3 spike.py");
        std::process::exit(status.code().unwrap_or(-1));
    }

## local_cargo.sh

    #!/usr/bin/env bash
    set -e

    HERE="$(cd "$(dirname "$0")" && pwd)"
    if [ -d "$HERE/.venv/bin" ]; then
        export PATH="$HERE/.venv/bin:$PATH"
    fi

    if [ "$1" = "run" ] || [ "$1" = "r" ]; then
        shift
        cd "$HERE/orchestrator"
        exec cargo run -- "$@"
    fi

    exec cargo "$@"

`chmod +x` it once.

The `.venv/bin` shim is what makes an activated python venv win over
system python. If you don't create one, it does nothing.

## Setup once

From the spike folder:

    python3 -m venv .venv
    .venv/bin/pip install -r requirements.txt
    .venv/bin/playwright install chromium

`requirements.txt` is just `playwright`.

## Running it

From the spike folder:

    c r

Args after `c r` go to `spike.py`. If the spike opens a URL, pass it:

    c r --url http://127.0.0.1:8080

Close the browser windows, press enter in the terminal, orchestrator
exits.

## Known gotchas

- Playwright's Chromium is not Brave or Edge. Two profiles of Chromium
  is how you simulate two devices. If a spike really needs Brave or
  Edge, Playwright can launch them with `channel=` — but for tester
  mechanics Chromium is fine.
- Screen size from `tkinter`: `root.winfo_screenwidth()` and
  `winfo_screenheight()`. Built-in, no dep.
- Two separate `browser.launch()` calls = two OS windows. Two `context`
  under one launch = one window. Use launch, not context, for
  side-by-side.
- `no_viewport=True` on the context, otherwise Playwright overrides
  `--window-size`.
