#!/usr/bin/bash

# fail fast
#
set -e

# print each command before it's executed
#
set -x

export RUSTFLAGS="-D warnings"

wasm-pack test  --firefox --headless
wasm-pack test  --chrome  --headless
wasm-pack test  --firefox --headless --release
wasm-pack test  --chrome  --headless --release
