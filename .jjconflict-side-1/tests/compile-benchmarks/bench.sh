#!/bin/bash

GIT_SHA=$(git show -s --format=%H >2 /dev/null || jj log -r @- --template "commit_id" --no-graph)
RUST_VERSION=$(rustc --version | cut -f2 -d" ")
DATE=$(date +%F)
TIMINGS_DIR=$(pwd)/timings

pushd github-query
cargo build
cargo build --release
hyperfine --export-json "$TIMINGS_DIR/github-query/$DATE.$GIT_SHA.$RUST_VERSION.debug.json" -p "cargo clean -p queries" "cargo build -p queries"
hyperfine --export-json "$TIMINGS_DIR/github-query/$DATE.$GIT_SHA.$RUST_VERSION.release.json" -p "cargo clean --release -p queries" "cargo build --release -p queries"
popd

cargo build -p cynic-parser
cargo build -p cynic-parser --release
hyperfine --export-json "$TIMINGS_DIR/cynic-parser/$DATE.$GIT_SHA.$RUST_VERSION.debug.json" -p "cargo clean -p cynic-parser" "cargo build -p cynic-parser"
hyperfine --export-json "$TIMINGS_DIR/cynic-parser/$DATE.$GIT_SHA.$RUST_VERSION.release.json" -p "cargo clean --release -p cynic-parser" "cargo build --release -p cynic-parser"
hyperfine --export-json "$TIMINGS_DIR/cynic-parser/$DATE.$GIT_SHA.$RUST_VERSION.debug.clean.json" -p "cargo clean" "cargo build -p cynic-parser"
hyperfine --export-json "$TIMINGS_DIR/cynic-parser/$DATE.$GIT_SHA.$RUST_VERSION.release.clean.json" -p "cargo clean --release" "cargo build --release -p cynic-parser"
