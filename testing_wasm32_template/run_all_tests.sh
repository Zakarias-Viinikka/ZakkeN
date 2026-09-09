#!/bin/bash

cargo test

echo ""
echo "Running wasm tests with Node..."
wasm-pack test --node
