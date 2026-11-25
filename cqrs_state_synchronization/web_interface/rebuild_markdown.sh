#!/usr/bin/env bash
# Rebuilds markdown_js/src/editor.js into markdown_web_output/cm_bridge.js (esbuild, via npm).
# Run it after any change in markdown_js/src/.
set -euo pipefail

cd "$(dirname "$0")/markdown_js"

if [ ! -d node_modules ]; then
    npm install
fi

npm run build
