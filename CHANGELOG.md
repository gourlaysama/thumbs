# Changelog

<!-- next-header -->
## [Unreleased] - TBD


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
[Unreleased]: https://github.com/gourlaysama/thumbs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/gourlaysama/thumbs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gourlaysama/thumbs/compare/01aa716...v0.1.0
