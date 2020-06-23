#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

cd cynic-book

wget https://github.com/rust-lang/mdBook/releases/download/v0.3.7/mdbook-v0.3.7-x86_64-unknown-linux-gnu.tar.gz
tar xvzf mdbook-v0.3.7-x86_64-unknown-linux-gnu.tar.gz
./mdbook build
