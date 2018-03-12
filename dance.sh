#!/bin/bash

set -e


# rustfmt seems to flip-flop on abc_generation/mod.rs
# running twice somewhat stabilizes this
cargo +nightly fmt
cargo +nightly fmt
cargo +nightly clippy
cargo test
