# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## v0.11.0 - 2026-02-27

### Changes

- update links to point at codeberg ([#1194](https://codeberg.org/obmarg/cynic/pulls/1194))
- bump to 1.85, edition 2024 ([#1172](https://codeberg.org/obmarg/cynic/pulls/1172))

## v0.8.4 - 2024-11-20

### New Features

- `Vec<T>::deserialize` now handles null coercion ([#1100](https://codeberg.org/obmarg/cynic/pulls/1100))

## v0.8.3 - 2024-11-12

### New Features

- Implement `rename_all` in derive(ValueDeserialize) ([#1094](https://codeberg.org/obmarg/cynic/pulls/1094))

### Fixes

- Better errors on misuse of the derive

## v0.8.2 - 2024-11-12

### Bug Fixes

- Option fields now always default to None if they are missing ([#1092](https://codeberg.org/obmarg/cynic/pulls/1092))

## v0.8.1 - 2024-11-12

- Initial Version
