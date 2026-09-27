# Changelog

All notable changes to FeROS Flasher are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Initial Rust command-line application with `list` and `flash` commands.
- Target resolution through versioned FeROS target manifests embedded at build
  time.
- RKNS image-signature, capacity, and SHA-256 validation.
- Native macOS device discovery through structured `diskutil` property lists.
- Startup-disk exclusion and whole, physical, writable, removable-media
  eligibility checks.
- Exact destructive-operation confirmation before privileged access.
- Raw-device writing, synchronization, byte-for-byte verification, and media
  ejection on macOS.
- Explicit unsupported host backends for Linux and Windows.
- Product-owned Makefile for staging native release executables under `bin/`.
- Product-owned technical specification, user guide, development guide, and
  architecture decision records.
- Native Linux, macOS, and Windows compilation validation through the reusable
  Flasher CI action.

### Verified

- Completed the macOS write-and-verify workflow using a removable microSD card
  through the built-in SDXC reader.
