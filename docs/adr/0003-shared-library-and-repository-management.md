# ADR 0003: Shared Library and Persistent Repository Management

## Status

Accepted — 2026-10-04

## Context

The CLI needs device-support repository registration and local selection while a
future GUI must reuse the same operations. The authoritative registry initially
lives in FeROS Targets and may later move to backend services. Product-specific
behavior must not depend on the workspace containing that repository.

## Decision

Expose media and repository operations through the Rust library. CLI prompts,
confirmation, and presentation remain frontend responsibilities. Media backends
receive a selected device and emit stage events. macOS sudo authorization remains
a terminal dependency until a graphical privilege mechanism is implemented.

Organize repository types, lookup, and persistence in `src/repository/repository.rs`,
application operations in `management.rs`, and reexports in `mod.rs` so clients call
`repository::add`, `available`, `current`, `select`, and `delete`.

The private registry loader fetches a fixed HTTPS endpoint on demand. Its version-1
TOML has `schema_version = 1` and `[[repositories]]` entries with `name`, `url`, and
`published_date`. Lookup returns the owned remote entry or no match. Network and
format errors remain errors. Only membership in this trusted registry makes a
repository official; remote self-declaration and the chosen local name do not.
The source loader can later be replaced by a backend-service adapter.

Persist local entries in `config/repositories.toml` beside the executable. Each
entry has a name, normalized URL, classification, default flag, and publication
date. Missing dates use current UTC registration time in this release. Registration
checks duplicate URLs and case-insensitive names. Selecting a default clears other
default flags; deletion rejects the current default. Numbers refer to current list
order and are not stable identifiers. Writes use a temporary file in the same
directory followed by atomic replacement.

Repository management does not yet replace embedded target resolution. The X55
profile remains compiled into the executable. The future catalog distribution
layout, synchronization policy, and ecosystem-wide registry governance require a
separate ADR in the main FeROS workspace.

## Consequences

- CLI and future GUI share behavior rather than duplicating it.
- A fresh installation has no automatically configured or selected repository.
- Local listing, selection, and deletion work offline; registration currently
  needs the official remote registry. Registry caching remains future work.
- Configuration requires a writable executable directory and must accompany a
  moved executable. Debug and release binaries have different stores.
- Atomic replacement avoids partial TOML writes, but simultaneous writers are
  not coordinated; operation locking remains a future requirement.
- Registration checks the URL and registry membership, not catalog contents,
  publisher signatures, remote availability, or device compatibility.
- This product ADR records implemented behavior. It does not claim ecosystem
  policy, backend services, catalog updates, pruning, or GUI delivery are complete.
