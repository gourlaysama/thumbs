thumbs
-------------
thumbs is a commandline tool to remove the cached thumbnails for files.

It supports any desktop environnement that respects the
[Freedesktop Thumbnail Managing Standard][2], so at least modern versions of KDE
and Gnome.

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

### Building

thumbs is written in Rust, so you'll need to [install Rust][1] first. It
also requires Rust 2018 edition, which is currently limited to the nigthly or
beta channel or Rust.

```
git clone https://github.com/gourlaysama/thumbs
cd thumbs
cargo build --release
./target/release/thumbs --version
thumbs 0.1.0-pre-75cede9
```

[1]: https://www.rust-lang.org
[2]: https://specifications.freedesktop.org/thumbnail-spec/latest/

### License

thumbs is licensed under the Apache Licence, Version 2.0. See the NOTICE for details
and the LICENSE file for a copy of the license.