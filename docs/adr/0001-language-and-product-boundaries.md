# ADR 0001: FeROS Flasher Language and Product Boundaries

## Status

Accepted

## Context

FeROS needs a safe and repeatable way to install generated target images onto
physical media. The existing Python toolchain already owns image construction,
inspection, and validation. Extending that toolchain to perform destructive
device operations would mix artifact production with host-specific installation
and recovery responsibilities.

The Flasher must eventually support macOS, Linux, and Windows while presenting
the same target-oriented workflow and safety guarantees on every host. It must
also be suitable for both a command-line interface and a future desktop
application.

Physical-media operations cross a security boundary: an incorrect device
selection can destroy user data. The implementation therefore requires strict
typing, explicit error handling, controlled host backends, and a design that can
share verified write logic with a future graphical interface.

## Decision

FeROS Flasher will be an independent Rust product under `flasher/`.
Its README, specifications, operational guides, release procedures, and
product-specific ADRs will remain inside that directory so the product can be
promoted to an independent repository without reconstructing its documentation.

The Python FeROS Toolchain will build, inspect, and validate target images. It
will not write physical media. FeROS Flasher will discover eligible devices,
enforce destructive-operation safeguards, write validated images, verify the
written bytes, and provide recovery-oriented host workflows.

Host-specific behavior will remain behind an internal backend boundary. macOS
will be the first implemented backend. Linux and Windows will be added through
the same product contract without changing target-image generation.

The Flasher will use canonical target identifiers such as
`rk3566-powkiddy-x55`. Target metadata comes from versioned target manifests
rather than duplicated command-line assumptions.

## Alternatives Considered

- **Continue the Python media writer:** Reuses existing code but mixes the
  Toolchain and Flasher products and weakens the repository boundaries defined
  by the project foundation.
- **Shell scripts around `dd`:** Provide a fast prototype but make validation,
  cross-platform behavior, structured errors, testing, and future desktop reuse
  unnecessarily fragile.
- **Go:** Offers simple cross-platform binaries and strong tooling, but Rust
  better matches the Flasher's low-level safety requirements and provides a
  direct path to sharing core logic with a native desktop application.
- **Separate native implementation for each host:** Allows immediate use of
  host conventions but duplicates safety policy and makes behavior inconsistent
  across platforms.

## Consequences

- The repository gains a separately versioned Rust product under `flasher/`.
- The Flasher carries its own product documentation and ADR sequence.
- Building the Flasher requires a Rust toolchain until release binaries are
  distributed.
- Python image tooling and Rust media tooling require an explicit artifact and
  target-manifest contract.
- macOS support may ship before Linux and Windows, but unsupported hosts must
  fail explicitly rather than expose placeholder behavior as functional.
- Every destructive write requires whole-device validation, startup-disk
  exclusion, capacity checks, explicit confirmation, write verification, and a
  clear recovery result.
- The existing Python physical-media writer will be removed after the Rust
  implementation reaches functional parity.
- A future desktop Flasher must reuse the same Rust domain and safety logic
  rather than reimplementing device operations in the user interface.