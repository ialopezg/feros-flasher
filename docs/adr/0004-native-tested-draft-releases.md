# ADR 0004: Native Tested Draft Releases

## Status

Accepted — 2026-10-04

## Decision

Build each candidate natively on macOS ARM64, Linux x86_64, and Windows x86_64
from one resolved tag commit. Validate Rust formatting, compilation, tests, and
Clippy, then run nondestructive CLI regression checks against a copied binary.
Package only explicit product files, build identity, lockfile inventory, and
notices. Recheck the extracted archive payload and produce SHA-256 sidecars.

Use Python standard-library scripts to orchestrate Cargo and packaging on all
three hosts. Python is not an application runtime dependency. Dependabot tracks
Cargo dependencies and GitHub Actions. External release actions are pinned by SHA.

Workflow dispatch requires an existing numeric version tag matching Cargo.toml
and Cargo.lock. Validate tag identity again before creating a GitHub draft release.
Do not overwrite existing releases automatically. Publication remains a separate
maintainer action after native CI, notes, notices, and compatibility review.

## Consequences

Linux and Windows candidates provide repository management only; native compilation
does not qualify their unimplemented physical-media backends. Automated tests do
not invoke physical-device writes or require a live target registry. macOS media
qualification remains separate. Packages are unsigned and macOS is not notarized.
Release notes must state these limits. The workflow records candidate evidence;
a successful run is not proof that a target console boots.
