# thumbs

**thumbs** is a command line tool to manage the cached thumbnails for files:

- it can delete the thumbnail for a file, for example to force it to be regenerated,
- it can cleanup stale thumbnails as a whole, removing those for files that no longer exist.

It supports any desktop environment that respects the
[Freedesktop Thumbnail Managing Standard][1], so at least modern versions of KDE and Gnome, and probably others.

## Installation

Precompiled binaries are available on the [Release Page] for x86_64 Linux (statically compiled).

If you are a **Fedora** (33+) user, you can install thumbs with:

```sh
sudo dnf copr enable gourlaysama/thumbs
sudo dnf install thumbs
```

Otherwise you will need to [build from source](#building-from-source).

## Usage

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

- Deleting thumbnails for files that don't exist:

 ```sh
# use globs to include or exclude files, e.g. no removable media or mkv files
$ thumbs cleanup -g '!/run/media/*' '!*.mkv'
Found 753 thumbnail(s) to delete. Use '-v/--verbose' for details, or add '-f/--force' to delete them.
 ```

## Building from source

thumbs is written in Rust, so you need a [Rust install] to build it. thumbs compiles with
Rust 1.52 or newer.

```sh
$ git clone https://github.com/gourlaysama/thumbs -b v0.3.0
$ cd thumbs
$ cargo build --release
$ ./target/release/thumbs --version
thumbs 0.3.0
```

## TODO

- [x] Cleanup thumbnails for files that don't exist
- [ ] Cleanup thumbnails for files not accessed in `duration`
- [ ] Generate thumbnails for files?
- [ ] Find out which DE this works with
- [ ] Prompt when in a terminal instead of asking to re-run with `-f/-d`

#### License

<sub>
thumbs is licensed under the Apache License, Version 2.0. See the NOTICE for details
and the LICENSE file for a copy of the license.
</sub>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in thumbs by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
</sub>

[Release Page]: https://github.com/gourlaysama/thumbs/releases/latest
[Rust install]: https://www.rust-lang.org/tools/install
[1]: https://specifications.freedesktop.org/thumbnail-spec/latest/
