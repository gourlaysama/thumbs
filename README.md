unthumbnailer
-------------
unthumbnailer is a commandline tool to remove the cached thumbnails for files.

It supports any desktop environnement that respects the
[Freedesktop Thumbnail Managing Standard][2], so at least modern versions of KDE
and Gnome.

### Building

unthumbnailer is written in Rust, so you'll need to [install Rust][1] first. It
also requires Rust 2018 edition, which is currently limited to the nigthly or
beta channel or Rust.

```
git clone https://github.com/gourlaysama/unthumbnailer
cd unthumbnailer
cargo build --release
./target/release/unthumbnailer --version
unthumbnailer 0.1.0
```

[1]: https://www.rust-lang.org
[2]: https://specifications.freedesktop.org/thumbnail-spec/latest/