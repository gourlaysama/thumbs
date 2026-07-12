use anyhow::Result;
use bytesize::ByteSize;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_json::json;
use std::{borrow::Cow, os::unix::fs::MetadataExt, path::Path};
use thumbs_rs::ThumbnailCache;

use crate::show;

pub fn run(p: &Path, json: bool) -> Result<bool> {
    let c = ThumbnailCache::init()?;

    let thumbnails = c.find_thumbnails_for_file(p)?;
    let mut count = 0;

    for t in thumbnails {
        if count == 0 && !json {
            show!("Index    Status       Size        Path");
        }
        let size = if let Ok(m) = t.path().metadata() {
            if !json {
                format!("{}", ByteSize::b(m.size()).display().iec()).into()
            } else {
                format!("{}", m.size()).into()
            }
        } else {
            Cow::Borrowed("unknown")
        };
        let status = if let Ok(stale) = t.is_stale() {
            if stale { "stale" } else { "up-to-date" }
        } else {
            "unknown"
        };
        let path = t.path().display();

        if json {
            let json_output = json!({
                "size": size,
                "status": status,
                "path": path.to_string(),
            });
            println!("{json_output}");
        } else {
            show!("{count:5}    {status}   {size}   {path}",);
        }

        count += 1;
    }

    if count == 0 {
        show!("No thumbnails for this file.");
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn cache_run(json: bool) -> Result<bool> {
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

    if json {
        let locations: Vec<String> = c.cache_locations().map(|loc| loc.display().to_string()).collect();
        let json_output = json!({
            "thumbnail_count": count,
            "cache_size": size,
            "cache_locations": locations,
        });
        println!("{json_output}");
    } else {
        for loc in c.cache_locations() {
            show!("Cache location: {}", loc.display());
        }

        show!("Thumbnail count: {count}");
        show!("Total cache size: {}", ByteSize::b(size).display().iec());
    }

    Ok(count != 0)
}
