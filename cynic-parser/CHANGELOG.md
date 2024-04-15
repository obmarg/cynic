# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased - xxxx-xx-xx

## [0.2.4](https://github.com/obmarg/cynic/compare/cynic-parser-v0.2.3...cynic-parser-v0.2.4) - 2024-04-15

### Fixed
- TypeSystemDocument::definitions() now returns an ExactSizeIterator ([#919](https://github.com/obmarg/cynic/pull/919))

## [0.2.3](https://github.com/obmarg/cynic/compare/cynic-parser-v0.2.2...cynic-parser-v0.2.3) - 2024-04-15

### Fixed
- Pretty printing output is now significantly improved, but still not perfect
  ([#916](https://github.com/obmarg/cynic/pull/916))

## v0.2.2 - 2024-04-12

### Fixes

- Pretty printing will now escape strings

## v0.2.1 - 2024-04-04

### Changes

- Some internal changes

## v0.2.0 - 2024-03-25

### Features

- Added executable parsing support
- String escaping is now supported properly.

### Changes

- Probably a ton of other things.  I'm not going to list the changes
  exhaustively because of how early this library is - I'd be surprised
  if anyone was using it (if I'm wrong please let me know).

## v0.1.0 - 2024-01-23

- Initial version.
- Supports parsing & printing GraphQL schemas
- Is fast
