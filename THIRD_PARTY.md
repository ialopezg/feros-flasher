# Third-party dependencies

FeROS Flasher is licensed under MIT. Rust dependencies retain their own licenses.
`Cargo.lock` records exact dependency versions and source checksums. Inspect the
license metadata and license files for all dependencies included in a distribution.

Release archives contain the project license, this notice, `Cargo.lock`, and a
`BUILD-INFO.json` inventory derived from the lockfile. That inventory includes
build and target-specific dependencies; it is not a complete binary SBOM and
must not be interpreted as a list of code linked into every native artifact.

The packaging script uses only the Python standard library. Python, Cargo, and
Rust tooling are build prerequisites and are not bundled as application runtimes.
The application is built with Cargo, not PyInstaller.

The built-in target profile is metadata. No Rockchip firmware, prepared images,
external target catalogs, or third-party games are included in these packages.

Before public distribution, review dependency license obligations and include any
required third-party notices or license texts in `licenses/`; the packaging script
copies that directory when present. License review is a maintainer publication gate.
