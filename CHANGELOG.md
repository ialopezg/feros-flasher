# Changelog

All notable changes to the Flasher are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [0.1.1] - 2026-09-27

### Added

- Added `help`, contextual help, and `version` commands.
- Added commit, build time, and host metadata to release binaries.

### Changed

- Unified help and version output under the product header.
- Updated SHA-256 handling for `sha2` 0.11.

## [0.1.0] - 2026-09-27

### Added

- Added the Rust `list` and `flash` commands.
- Added target profiles and image validation.
- Added safe macOS device discovery, writing, verification, and ejection.
- Added Linux and Windows placeholder backends.
- Added product documentation, release builds, and cross-platform CI.

### Verified

- Verified physical microSD writing on macOS.
