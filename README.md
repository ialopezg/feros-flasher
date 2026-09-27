# FeROS Flasher

[![CI](https://github.com/ialopezg/feros-flasher/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ialopezg/feros-flasher/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-supported-2ea44f?style=flat-square&logo=apple&logoColor=white)](docs/user-guide.md)
[![Linux](https://img.shields.io/badge/Linux-build%20verified-0969da?style=flat-square&logo=linux&logoColor=white)](docs/specification.md)
[![Windows](https://img.shields.io/badge/Windows-build%20verified-0969da?style=flat-square&logo=windows11&logoColor=white)](docs/specification.md)

[![User Guide](https://img.shields.io/badge/User%20Guide-read-0969da?style=flat-square)](docs/user-guide.md)
[![Development](https://img.shields.io/badge/Development-build-8250df?style=flat-square)](docs/development.md)
[![Specification](https://img.shields.io/badge/Specification-review-1f883d?style=flat-square)](docs/specification.md)
[![ADR](https://img.shields.io/badge/ADR-decisions-d1242f?style=flat-square)](docs/adr/)
[![CHANGELOG](https://img.shields.io/badge/Changelog-history-6e7781?style=flat-square)](CHANGELOG.md)

FeROS Flasher safely installs and verifies FeROS target images on physical
media. It is an independent Rust product and does not build target images.

## Current Status

- Project horizon: Horizon 1 — Foundation
- Product milestone: initial macOS CLI
- Supported host: macOS
- Supported target: `rk3566-powkiddy-x55`
- Linux support: not implemented
- Windows support: not implemented

The Flasher is experimental. Review the selected device carefully before
confirming a write.

## Responsibilities

FeROS Flasher owns:

- physical-device discovery;
- startup-disk exclusion;
- removable-media eligibility checks;
- image capacity and target-signature checks;
- explicit destructive-operation confirmation;
- physical-media writing;
- byte-for-byte verification;
- media ejection and clear operation results.

FeROS Toolchain owns:

- source builds;
- artifact inspection;
- RKNS construction;
- complete target-image validation.

## Build

```sh
cargo build --manifest-path flasher/Cargo.toml
```

## List Eligible Devices

```sh
cargo run --manifest-path flasher/Cargo.toml -- list
```

## Flash the PowKiddy X55 Image

```sh
cargo run --manifest-path flasher/Cargo.toml -- \
  flash \
  --target rk3566-powkiddy-x55 \
  --image build/x55/boot/feros-x55.img
```

The interactive flow displays eligible media and requires an exact confirmation
phrase before writing. Run the Flasher as a normal user. It invokes `sudo` only
for raw-device access after confirmation.

## Safety Model

The macOS backend rejects:

- the current startup disk;
- partitions instead of whole disks;
- nonphysical devices;
- read-only media;
- fixed internal media that is not removable;
- media smaller than the selected image;
- images without the expected target signature.

The initial implementation never formats media or modifies partition tables
separately. It writes the selected image from byte zero and verifies exactly the
number of bytes contained in the source image.

---
## Contributing to FeROS Flasher

Thank you for your interest in contributing to **FeROS**.

FeROS Flasher is part of the **FeROS ecosystem** and provides foundational packages shared across Entiqon projects.

- **Original Author:** Isidro A. López G.
- **Organization:** FeROS Project
- **Official Repository:** https://github.com/ialopezg/feros

Ideas, experiments, technical review, and contributions are welcome.

---

## License

**FeROS Flasher** is licensed under the [MIT License](LICENSE).
