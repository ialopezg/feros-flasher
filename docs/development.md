# FeROS Flasher Development and Release Guide

## Purpose

This guide defines how maintainers validate, build, and stage FeROS Flasher
executables. It is not an end-user flashing guide.

## Prerequisites

- A stable Rust toolchain managed by `rustup`.
- Cargo, Rustfmt, and Clippy.
- GNU Make.
- The native host tools required by the implemented backend.

The macOS release must be built and physically validated on macOS. Linux and
Windows placeholders may compile, but they must not be published as functional
flashers until their native backends pass physical-media verification.

## Validation

Run from `flasher/`:

```sh
make fmt
make check
make test

cargo clippy \
  --manifest-path Cargo.toml \
  --all-targets \
  -- \
  -D warnings
```

All commands must complete without warnings or failures.

## Release Build

```sh
make release
```

Cargo retains its internal build artifacts in `flasher/target/`. The flasher
Makefile publishes only the final native executable under:

```text
bin/<host>/<architecture>/flasher[.exe]
```

Current macOS ARM64 output:

```text
bin/darwin/aarch64/flasher
```

Windows executables use the `.exe` suffix. Darwin and Linux executables do not.

The repository-local `bin/` directory contains generated artifacts and is
ignored by Git. Workspace integration may override `BIN_ROOT` to use the
workspace's shared `bin/` directory.

`Cargo.lock` is versioned because FeROS Flasher is an application.

## Release Verification

Verify the staged executable:

```sh
file ../bin/darwin/aarch64/flasher
../bin/darwin/aarch64/flasher --version
../bin/darwin/aarch64/flasher --help
../bin/darwin/aarch64/flasher list
```

Confirm that the published executable matches the Cargo release artifact:

```sh
cmp \
  target/release/flasher \
  ../bin/darwin/aarch64/flasher
```

Generate a release checksum when packaging an artifact:

```sh
shasum -a 256 ../bin/darwin/aarch64/flasher
```

## Physical Release Qualification

A functional host release requires evidence for all of the following:

- the startup disk is excluded;
- an eligible removable whole disk is discovered;
- an invalid target is rejected;
- an invalid image is rejected before elevation;
- a missing or incorrect confirmation aborts safely;
- the correct device is unmounted;
- the image is written to the raw whole-device path;
- the written byte range compares successfully with the source image;
- the device is ejected;
- no unsupported host is presented as functional.

## Versioning

Update the package version in `flasher/Cargo.toml` only as part of an approved
product release. Regenerate and commit `flasher/Cargo.lock`, run the complete
validation suite, and build the release from the reviewed commit.

## Cleaning

```sh
make clean
```

This removes the Cargo build artifacts under `target/`. It does not remove
executables already staged under the configured binary output directory.

## Future Distribution Requirements

Before public macOS distribution, define and automate code signing,
notarization, archive naming, checksums, provenance, and release attachment
generation. Equivalent native trust and packaging requirements must be defined
before Linux or Windows releases are published.
