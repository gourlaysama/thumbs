use anyhow::Result;
use globset::GlobSet;
use std::io::{IsTerminal, stdin};
use thumbs_rs::ThumbnailCache;

use crate::{cached_delete, interactive::user_prompt, show};

pub fn run(force: bool, exclude: &GlobSet, include: &GlobSet) -> Result<bool> {
    let c = ThumbnailCache::init()?;
    let thumbs: Vec<_> = c
        .search_thumbnails(exclude, include, true)
        .collect();
    let nb_thumbs = thumbs.len();

    if nb_thumbs == 0 {
        show!("Found no thumbnails to cleanup.")
    } else if !force {
        if stdin().is_terminal() {
            return user_prompt(&thumbs, || cached_delete(&thumbs));
        } else {
            show!(
                "Found {nb_thumbs} thumbnail(s) to delete. Use '-v' for details, or '-f/--force' to delete them."
            );
        }
    } else {
        cached_delete(&thumbs);
    }

    Ok(nb_thumbs != 0)
}
