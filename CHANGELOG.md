# Changelog

**thumbs** is a CLI tool to manage the cached thumbnails for files.

<!-- next-header -->
## [Unreleased] - TBD


## [0.2.2] - 2021-07-09

### Changed

* Passing a directory to `delete` is now supported: the deletion applies to its content, but `--recursion/-r` is still needed to go deeper in the directory hierarchy.

### Fixed

* `locate` accidentally took multiple paths, like `delete`. It now only take a single path (which doesn't have to exist).


## [0.2.1] - 2021-07-07

### Fixed

* The `locate` command did not print the located path when run without `-vv`.

## [0.2.0] - 2021-07-07

### Packaging

* A statically-built binary (built with musl) is now available for every release.
* The Minimum Supported Rust Version is 1.52.

### Added

* New `cleanup` command that deletes thumbnails for files that no longer exist, with a `--glob/-g` option to include/exclude paths from the cleanup (the default is to include everything). Actual deletion happens with `--force/-f`, otherwise it just prints what it found. Exclusions are prefixed with a `!` character (e.g. `--glob '!/run/media/**'`).

## [0.1.0] - 2018-11-15

### Added

* `delete` command to delete the thumbnail for files,
* `locate` command to print the path to a thumbnail for a file.

<!-- next-url -->
[Unreleased]: https://github.com/gourlaysama/thumbs/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/gourlaysama/thumbs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/gourlaysama/thumbs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/gourlaysama/thumbs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gourlaysama/thumbs/compare/01aa716...v0.1.0
