use anyhow::Result;
use bytesize::ByteSize;
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
