# FeROS Flasher Technical Specification

## Status

- Specification version: 1
- Product stage: Stage 1
- Implemented host: macOS
- Placeholder hosts: Linux and Windows
- Initial target: `rk3566-powkiddy-x55`

## Purpose

FeROS Flasher is the host-side application responsible for installing a
prepared FeROS image on physical target media and verifying the written bytes.
It does not build FeROS, construct RKNS images, partition media, or format
filesystems.

## Product Boundary

The FeROS Toolchain owns source compilation, artifact inspection, image
construction, and complete image-format validation. FeROS Flasher consumes a
prepared image and owns physical-device discovery, safety validation,
confirmation, writing, verification, and ejection.

The root `../targets` manifests are the source of truth for target identity,
image format, artifact locations, and media validation metadata. The flasher
embeds supported manifests at compile time so the executable does not depend on
the repository working directory at runtime.

## Command-Line Contract

```text
flasher list
flasher flash --target <target-id> --image <path> [--device <whole-device>]
```

`list` must not modify storage. `flash` must not perform a destructive action
until image validation, device validation, refreshed device discovery, and
exact user confirmation have succeeded.

## Safety Invariants

The implementation must:

1. run as a normal user and request elevation only for raw-device access;
2. reject the current host startup disk;
3. reject partitions and accept only whole physical disks;
4. reject read-only media;
5. reject fixed internal media that is not removable;
6. reject media smaller than the selected image;
7. reject images whose target signature is absent at the manifest offset;
8. refresh the selected device before confirmation and again before writing;
9. require the exact phrase `WRITE <device-path>`;
10. verify exactly the number of bytes contained in the source image;
11. report cancellation without claiming that a device was modified.

## Darwin Backend

The Darwin backend uses property-list output from `diskutil` rather than
parsing localized human-readable output.

Discovery uses `diskutil list -plist physical`, preferring the `WholeDisks`
property and retaining `AllDisksAndPartitions` as a compatibility fallback.
Per-device validation uses `diskutil info -plist <device>`.

The current write sequence is:

1. `diskutil unmountDisk <device>`;
2. `sudo dd if=<image> of=<raw-device> bs=1048576`;
3. `sync`;
4. `sudo cmp -n <image-size> <image> <raw-device>`;
5. `diskutil eject <device>`.

The raw device path is derived from the validated whole-disk identifier. For
example, `/dev/disk4` becomes `/dev/rdisk4`.

## Target Manifest Contract

The flasher currently consumes these manifest values:

- `target.id`;
- `target.manifest_version`;
- `board.manufacturer`;
- `board.name`;
- `build.image_format`;
- `media.signature_offset`;
- `media.signature`;
- `media.write_offset`.

Manifest version 1 supports only `image_format = "rkns"` and
`media.write_offset = 0` in the current implementation.

## Host Backend Contract

Each host module exposes the same package-level factory:

```rust
pub(super) fn backend() -> Result<Box<dyn MediaBackend>>
```

The dispatcher selects `darwin::backend()`, `linux::backend()`, or
`windows::backend()` at compile time. Linux and Windows intentionally return a
clear unsupported error for media operations until their native backends are
implemented and physically verified.

## Verification Record

The initial Darwin implementation has been physically verified with a built-in
SDXC reader. The verified sequence covered device discovery, RKNS signature
validation, explicit confirmation, unmounting, raw-device writing, synchronization,
byte-for-byte comparison, and media ejection.

## Known Limitations

- Linux and Windows physical-media operations are not implemented.
- Darwin currently delegates privileged raw-device operations to `sudo`.
- The write operation does not yet expose progress reporting.
- Release binaries are not yet code-signed or notarized.
- Only target-manifest version 1 and RKNS images are accepted.
