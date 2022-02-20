#!/bin/bash

cd github-query
cargo build 
cargo build --release

GIT_SHA=$(git show -s --format=%H)
RUST_VERSION=$(rustc --version | cut -f2 -d" ")
DATE=$(date +%F)

hyperfine --export-json "../timings/github-query/$DATE.$GIT_SHA.$RUST_VERSION.debug.json" -p "cargo clean -p queries" "cargo build -p queries" 
hyperfine --export-json "../timings/github-query/$DATE.$GIT_SHA.$RUST_VERSION.release.json" -p "cargo clean --release -p queries" "cargo build --release -p queries" 
