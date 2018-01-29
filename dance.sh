#!/bin/bash

# rustfmt seems to flip-flop on abc_generation/mod.rs - running twice somewhat stabilizes this
rustfmt src/lib.rs && rustfmt src/lib.rs && cargo +nightly clippy && cargo test

