thumbs [![Build Status](https://travis-ci.org/gourlaysama/thumbs.svg?branch=master)](https://travis-ci.org/gourlaysama/thumbs)
-------------
thumbs is a command line tool to manage the cached thumbnails for files.

It supports any desktop environment that respects the
[Freedesktop Thumbnail Managing Standard][2], so at least modern versions of KDE
and Gnome, and probably others.

 - Deleting thumbnails:

```sh
$ thumbs delete MyMovie.mkv MyImage.png
Deleted 2 thumbnail(s).

$ thumbs delete --dry-run --recursive ~/Videos/
Found 316 thumbnail(s) to delete. Use '-v/--verbose' for details, or remove '-d/--dry-run' to delete them.

```

 - Locating thumbnails, useful for scripting:

```sh
$ thumbs locate MyMovie.mkv
/home/me/.cache/thumbnails/large/b94bf1a19b509a749d34e836a29d61c5.png

$ cp `thumbs locate MyMovie.mkv | head -1` MyMovie_thumbnail.png

```

 - Deleting thumbnails for files that don't exist (on `master` only):

 ```sh
# use globs to include or exclude files, e.g. no removable media or mkv files
$ thumbs cleanup -g '!/run/media/*' '!*.mkv'
Found 753 thumbnail(s) to delete. Use '-v/--verbose' for details, or add '-f/--force' to delete them.
 ```

 - TODO:
   - [x] Cleanup thumbnails for files that don't exist
   - [ ] Cleanup thumbnails for files not accessed in `duration`
   - [ ] Generate thumbnails for files?
   - [ ] Find out which DE this works with
   - [ ] Prompt when in a terminal instead of asking to re-run with `-f/-d`

### Installation

For x86_64 Linux, download the binary from the [v0.1.0 release page][3]. Otherwise, see below for building instructions.

### Building

thumbs is written in Rust, so you'll need to [install Rust][1] first. It
also requires Rust 2018 edition, which is currently limited to the nightly or
beta channel of Rust. Then:

```sh
$ git clone https://github.com/gourlaysama/thumbs -b v0.2.0
$ cd thumbs
$ cargo build --release
$ ./target/release/thumbs --version
thumbs 0.2.0
```

thumbs requires (by default) ImageMagick 6.9. You can build with ImageMagick 7 instead using:
```sh
cargo build --no-default-features --features cleanup-magick7 --release
```

 - ImageMagick 6:
   - Ubuntu 18.04+: `imagemagick-6.q16`, and `libmagickwand-dev` for building
   - Debian Stretch+: `imagemagick-6.q16`, and `libmagickwand-dev` for building
   - Fedora 27+: `ImageMagick`, and `ImageMagick-devel` for building
   - Archlinux: `imagemagick6` in Extra
   
 - ImageMagick 7:
   - Archlinux: `imagemagick` in Extra

Or you can disable the cleanup feature entirely:
```sh
cargo build --no-default-features --release
```

### License

thumbs is licensed under the Apache License, Version 2.0. See the NOTICE for details
and the LICENSE file for a copy of the license.

[1]: https://www.rust-lang.org
[2]: https://specifications.freedesktop.org/thumbnail-spec/latest/
[3]: https://github.com/gourlaysama/thumbs/releases/tag/v0.1.0