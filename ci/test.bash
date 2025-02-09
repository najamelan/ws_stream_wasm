#!/usr/bin/bash

# fail fast
#
set -e

# print each command before it's executed
#
set -x

export RUSTFLAGS="-D warnings --cfg getrandom_backend=\"wasm_js\""

wasm-pack test  --firefox --headless -- --all-features
wasm-pack test  --chrome  --headless -- --all-features
wasm-pack test  --firefox --headless -- --all-features --release
wasm-pack test  --chrome  --headless -- --all-features --release
