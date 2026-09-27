# ADR 0002: Independent FeROS Flasher Repository

## Status

Accepted

## Context

FeROS Flasher began inside the FeROS monorepository so its safety model,
target contract, and initial Darwin backend could mature beside FeROS Core.
The product now owns its implementation, documentation, validation action,
release process, and version history.

Keeping the product embedded would couple Flasher releases to FeROS Core
changes and make independent distribution harder. It would also require the
Flasher build to retain paths that only exist in the monorepository.

## Decision

FeROS Flasher will move to an independent repository while remaining part of
the FeROS ecosystem.

The Flasher repository will own:

- Rust source code and dependencies;
- product documentation and ADRs;
- continuous integration and release procedures;
- Flasher compatibility profiles under `../../targets`;
- Flasher changelog, tags, and release versions.

FeROS Core will continue to own build manifests, firmware inputs, target-image
construction, and generated images. Flasher compatibility profiles contain
only the metadata required to identify and safely install a prepared image.
They are not FeROS Core build manifests.

The two products will version and release independently. Integration occurs
through canonical target identifiers and the prepared-image contract, not
through repository-relative paths.

## Consequences

- The Flasher can be built, tested, tagged, and released without FeROS Core.
- Its CI workflow and reusable validation action move with the product.
- Generated executables are staged under the repository-local `bin/` tree.
- FeROS Core documentation links to the independent Flasher repository.
- Compatibility-profile changes must be reviewed against the corresponding
  released FeROS target contract.
- Removing the embedded `flasher/` directory becomes a separate Core change
  performed only after the independent repository has been validated.
