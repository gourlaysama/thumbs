use anyhow::Result;
use bytesize::ByteSize;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::{borrow::Cow, os::unix::fs::MetadataExt, path::Path};
use thumbs_rs::ThumbnailCache;

use crate::show;

pub fn run(p: &Path) -> Result<bool> {
    let c = ThumbnailCache::init()?;

    let thumbnails = c.find_thumbnails_for_file(p)?;
    let mut count = 0;

    for t in thumbnails {
        if count == 0 {
            show!("Index    Status       Size        Path");
        }
        let size = if let Ok(m) = t.path().metadata() {
            format!("{}", ByteSize::b(m.size()).display().iec()).into()
        } else {
            Cow::Borrowed("unknown")
        };
        let status = if let Ok(stale) = t.is_stale() {
            if stale { "stale" } else { "up-to-date" }
        } else {
            "unknown"
        };

        show!("{count:5}    {status}   {size}   {}", t.path().display());

        count += 1;
    }

    if count == 0 {
        show!("No thumbnails for this file.");
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn cache_run() -> Result<bool> {
    let c = ThumbnailCache::init()?;

    let mut size = 0;
    let mut count: usize = 0;

    let mut builder_include = GlobSetBuilder::new();
    builder_include.add(Glob::new("**")?);

    for t in c.search_thumbnails(&GlobSet::empty(), &builder_include.build()?, false) {
        count += 1;
        if let Ok(m) = t.path().metadata() {
            size += m.size();
        }
    }

    for loc in c.cache_locations() {
        show!("Cache location: {}", loc.display());
    }

    show!("Thumbnail count: {count}");
    show!("Total cache size: {}", ByteSize::b(size).display().iec());

    Ok(count != 0)
}
