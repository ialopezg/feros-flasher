# FeROS Flasher

FeROS Flasher safely installs and verifies FeROS target images on physical
media. It is an independent Rust product and does not build target images.

## Current Status

- Project horizon: Horizon 1 â€” Foundation
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