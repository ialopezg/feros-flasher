# Security Policy

## Reporting privately

Report suspected vulnerabilities through GitHub Private Vulnerability Reporting:

https://github.com/ialopezg/feros-flasher/security/advisories/new

The maintainer must enable and verify this feature. This file does not enable
it. If the form is unavailable, request a private contact channel without
publishing vulnerability details. No security email address is designated.
Do not post secrets, personal device dumps, or exploit details in public issues.

Include the affected version or commit, host OS and architecture, impact, and
minimal reproduction using synthetic files where possible. Do not repeat a
physical-media write solely to reproduce a report. No bounty or response-time
commitment is offered.

## Supported versions

During 0.x, fixes target main and the latest published release. Older releases
receive no routine backports; a fix may require upgrading.

## Trust boundaries

Flasher can overwrite physical media. Selection, confirmation, and verification
reduce mistakes but do not authenticate image authors or establish payload safety.
Use only images and devices you are authorized to handle.

HTTPS URL validation and recognition in the official repository registry do not
constitute a security audit of repository content. Repository management does
not yet download device catalogs or change the embedded flashing target.
Configuration is stored beside the executable in config/repositories.toml;
protect that directory from untrusted modification. Concurrent configuration
writes are not coordinated by a lock.

## Distribution

Release automation creates drafts for maintainer review. Adjacent SHA-256 files
detect accidental corruption; replacement of both archive and checksum is not
prevented. Packages do not claim code signing, notarization, reproducible builds,
or provenance attestations. See RELEASE.md and THIRD_PARTY.md for review limits.
