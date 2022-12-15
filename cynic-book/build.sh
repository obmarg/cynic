#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

cd cynic-book

wget https://github.com/rust-lang/mdBook/releases/download/v0.4.15/mdbook-v0.4.15-x86_64-unknown-linux-gnu.tar.gz
tar xvzf mdbook-v0.4.15-x86_64-unknown-linux-gnu.tar.gz

# mkdir -p mdbook-linkcheck && cd "$_" && \
  # curl -L https://github.com/Michael-F-Bryan/mdbook-linkcheck/releases/latest/download/mdbook-linkcheck.x86_64-unknown-linux-gnu.zip -o mdbook-linkcheck.zip && \
  # unzip "$_" && \
  # chmod +x mdbook-linkcheck && \
  # export PATH=$PWD:$PATH && \
  # cd ..

rustup default stable


if ! command -v mdbook-linkcheck &> /dev/null
then
	cargo install mdbook-linkcheck
fi

./mdbook build
