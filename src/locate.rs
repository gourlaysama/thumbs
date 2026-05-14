use std::path::Path;

use anyhow::Result;
use log::*;
use thumbs_rs::ThumbnailCache;

pub fn run(file: &Path) -> Result<bool> {
    let cache = ThumbnailCache::init()?;
    let thumbnails = cache.find_thumbnails_for_file(file)?;
    let mut empty = true;

    for t in thumbnails {
        empty = false;
        println!("{}", t.path().display());
    }

    if empty {
        warn!("Found no thumbnails.");
    }

    Ok(!empty)
}
