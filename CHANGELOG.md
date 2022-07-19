# Changelog

**thumbs** is a CLI tool to manage the cached thumbnails for files.
<!-- next-header -->
## [Unreleased] - TBD

## [0.4.5] - 2022-07-19

### Packaging

* Shell completions for thumbs are now provided (for bash, zsh and fish).

## [0.4.4] - 2022-06-30

### Packaging

* There is now a man page for thumbs available in the release tarball (generated with pandoc, from `doc/thumbs.1.md`).

## [0.4.3] - 2022-06-22

### Packaging

* There is now a x86_64 debian package available for download on the release page.

## [0.4.2] - 2022-05-16

### Fixes

* Added the Nautilus integration script to the release tarball.

## [0.4.1] - 2022-05-16

### Features

* Added Nautilus integration (Delete thumbnails from context menu in Nautilus). See `extra/nautilus/INSTALL.md` for instructions, or install the `thumbs-nautilus` package from COPR.

## [0.3.3] - 2022-03-23

### Features

* Support for deleting the placeholder thumbnails created when thumbnail generation fails.

## [0.4.0] - 2022-03-22

### Packaging

* The Minimum Supported Rust Version for thumbs is now 1.57.

### Features

* Support for deleting the placeholder thumbnails created when thumbnail generation fails.

## [0.3.2] - 2021-11-05

### Changes

* `--version` output now shows more build information.

## [0.3.1] - 2021-07-13

### Fixes

* Fixed publishing of binaries when a version is released.

## [0.3.0] - 2021-07-13

### Changes

* `delete` now requires `-f/--force` to actually delete thumbnails and will otherwise prompt (on tty).
* `delete` has no `-d/--dry-run` option anymore: it is now the default behavior.

### Features

* New `-l/--last-accessed <timestamp/duration>` option for `delete`. It allows deleting only thumbnails for file that have not been accessed since that time. The argument can be either a RFC3339-like time-stamp (`2020-01-01 11:10:00`) or a free-form duration like `1year 15days 1week 2min` or `1h 6s 2ms` that is taken from the current time.
* thumbs will now prompt for action when connected to a tty, with 3 options `y/N/d` for deletion, doing nothing, and printing the files whose thumbnails will be deleted, respectively.

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
[Unreleased]: https://github.com/gourlaysama/thumbs/compare/v0.4.5...HEAD
[0.4.5]: https://github.com/gourlaysama/thumbs/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/gourlaysama/thumbs/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/gourlaysama/thumbs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/gourlaysama/thumbs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/gourlaysama/thumbs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/gourlaysama/thumbs/compare/v0.3.2...v0.4.0
[0.3.3]: https://github.com/gourlaysama/thumbs/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/gourlaysama/thumbs/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/gourlaysama/thumbs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/gourlaysama/thumbs/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/gourlaysama/thumbs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/gourlaysama/thumbs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/gourlaysama/thumbs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gourlaysama/thumbs/compare/01aa716...v0.1.0
