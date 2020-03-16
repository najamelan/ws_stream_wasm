#!/usr/bin/bash

# Only run on nightly.

# fail fast
#
set -e

# print each command before it's executed
#
set -x

export RUSTFLAGS="-D warnings"


cargo doc --all-features
