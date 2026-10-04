# FeROS Flasher

[![CI](https://github.com/ialopezg/feros-flasher/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ialopezg/feros-flasher/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/ialopezg/feros-flasher?style=flat-square)](https://github.com/ialopezg/feros-flasher/releases)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

[![User Guide](https://img.shields.io/badge/User%20Guide-read-0969da?style=flat-square)](docs/user-guide.md)
[![Development](https://img.shields.io/badge/Development-build-8250df?style=flat-square)](docs/development.md)
[![ADR](https://img.shields.io/badge/ADR-decisions-d1242f?style=flat-square)](docs/adr)
[![Changelog](https://img.shields.io/badge/Changelog-history-6e7781?style=flat-square)](CHANGELOG.md)

FeROS Flasher installs prepared FeROS images on physical media and verifies the
written bytes. It is an independent Rust application with a shared library for
the CLI and a future graphical interface. FeROS Builder owns image construction.

## v0.2.0 capabilities

| Operation                            | macOS       | Linux           | Windows         |
|--------------------------------------|-------------|-----------------|-----------------|
| Help, version, repository management | Implemented | Implemented     | Implemented     |
| Physical-device listing and flashing | Implemented | Not implemented | Not implemented |

This is experimental software. Automated CLI checks do not establish physical
media qualification or successful boot on a target device.

## Build and run

From the Flasher repository root (`tools/flasher` in the FeROS workspace):

```sh
cargo build --release --locked
./target/release/flasher help
./target/release/flasher version
./target/release/flasher channel --list
```

On Windows, the executable is `target/release/flasher.exe`.

## Repositories

```sh
flasher channel
flasher channel --add
flasher channel --list
flasher channel --select 1
flasher channel --delete 2
```

`--add` asks for an HTTPS URL, an optional name, and whether to select it as the
default. A blank name uses the URL domain. Registration queries the official
FeROS Targets registry; it does not download targets or validate the remote
repository's catalog. An unlisted URL is saved as unofficial.

Configuration is stored in `config/repositories.toml` beside the executable.
A fresh installation has no configured repositories or selected default. Listing,
selection, and deletion work offline; adding currently requires access to the
official registry. The default repository cannot be deleted until another is selected.

The selected repository does not yet change flashing behavior: Flasher still
uses the embedded `rk3566-powkiddy-x55` target manifest. Repository updates,
pruning, catalog downloads, and registry caching are not implemented in v0.2.0.

## Physical media (macOS)

```sh
flasher list
flasher flash --target rk3566-powkiddy-x55 --image /path/to/prepared.img
```

Run as a normal user. Review the selected whole disk and image, then enter the
exact `WRITE /dev/diskN` phrase shown. The backend validates device eligibility,
capacity, and the image signature, refreshes device metadata, unmounts the disk,
writes through `sudo`, compares the written bytes, and ejects the disk.

Writing overwrites the image-sized region starting at byte zero. Signature and
hash checks do not authenticate firmware or prove that the device will boot.

## Project

- Original author: Isidro A. López G.
- Organization: FeROS Project
- Repository: https://github.com/ialopezg/feros-flasher
- [User guide](docs/user-guide.md), [development](docs/development.md),
  [specification](docs/specification.md)
- License: [MIT](LICENSE); dependency scope: [third-party notices](THIRD_PARTY.md)
