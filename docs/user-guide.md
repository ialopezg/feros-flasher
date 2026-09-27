# FeROS Flasher User Guide

## Support Status

FeROS Flasher currently supports physical-media writing on macOS. Linux and
Windows media operations are not supported yet.

Flashing overwrites the beginning of the selected storage device. Selecting the
wrong device can destroy data. Review every displayed field before confirming.

## List Eligible Devices

Insert the target microSD card and run:

```sh
bin/darwin/aarch64/flasher list
```

Example:

```text
FeROS Flasher: eligible physical disks:

  /dev/disk4 — Built In SDXC Reader — 14.5 GiB — Secure Digital
```

The macOS startup disk is excluded automatically. Only eligible whole physical
devices are displayed.

## Flash an Image

From the repository root:

```sh
bin/darwin/aarch64/flasher flash \
  --target rk3566-powkiddy-x55 \
  --image build/x55/boot/feros-x55.img
```

The flasher validates the target image and asks you to select an eligible
device. A specific device may be supplied explicitly:

```sh
bin/darwin/aarch64/flasher flash \
  --target rk3566-powkiddy-x55 \
  --image build/x55/boot/feros-x55.img \
  --device /dev/disk4
```

Always run `diskutil list` again before reusing a device identifier. macOS may
assign a different identifier after media is removed or reinserted.

## Confirmation

Before writing, the flasher displays:

- target identity;
- whole-device path;
- device model and protocol;
- capacity and removable status;
- canonical image path;
- image size;
- image SHA-256.

To proceed, enter the exact phrase shown by the flasher:

```text
WRITE /dev/disk4
```

Any other input aborts the operation:

```text
FeROS Flasher: operation aborted: required user confirmation was not provided; no storage device was modified
```

## Successful Operation

A successful operation reports:

```text
FeROS Flasher: unmounting /dev/disk4...
FeROS Flasher: writing ...
FeROS Flasher: verifying written bytes...
FeROS Flasher: ejecting /dev/disk4...

FeROS Flasher: target media written and verified successfully.
```

The flasher verifies the written byte range before reporting success.

## Troubleshooting

### No eligible disks were found

Confirm that macOS sees the card:

```sh
diskutil list
```

The device must be a writable, removable, whole physical disk and must not be
the macOS startup disk.

### The target is unsupported

Use the canonical target identifier:

```text
rk3566-powkiddy-x55
```

### The image is rejected

Rebuild and validate the prepared image with the FeROS Toolchain. The flasher
requires the signature and offset declared by the embedded target manifest.

### The device identifier changed

Run `diskutil list` again. Never assume that a previous `/dev/diskN` identifier
still refers to the same physical device.

### The password prompt appears

The Darwin backend invokes `sudo` only after target validation, device
validation, and exact confirmation. The password prompt authorizes raw-device
writing and verification.
