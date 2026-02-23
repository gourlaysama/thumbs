use std::path::Path;

use anyhow::Result;
use thumbs_rs::ThumbnailCache;

use crate::show;

pub fn run(file: &Path) -> Result<bool> {
    let cache = ThumbnailCache::init()?;
    let thumbnails = cache.find_thumbnails_for_file(&file)?;
    let mut empty = true;

    for t in thumbnails {
        empty = false;
        show!("{}", t.path().display());
    }
    Ok(!empty)
}
