use anyhow::Result;
use log::*;
use std::{
    io::{IsTerminal, stdin},
    path::PathBuf,
    time::SystemTime,
};
use thumbs_rs::ThumbnailCache;

use crate::interactive::user_prompt;
use crate::{cached_delete, show};

pub fn run(
    files: &[PathBuf],
    force: bool,
    last_accessed: Option<SystemTime>,
    recursive: bool,
    hidden: bool,
) -> Result<bool> {
    let c = ThumbnailCache::init()?;
    let results = c.find_thumbnails_for_files_in(files, last_accessed, recursive, hidden)?;
    let thumbnail_count = results.thumbnail_paths.len();

    if results.ignored_directories != 0 {
        info!(
            "Ignoring {} folder(s). Enable '-r/--recursive' to recurse into directories.",
            results.ignored_directories
        )
    }
    if thumbnail_count == 0 {
        warn!("Found no thumbnails.");
    } else if !force {
        if stdin().is_terminal() {
            return user_prompt(&results.thumbnail_paths, || {
                cached_delete(&results.thumbnail_paths);
            });
        } else {
            if log_enabled!(Level::Info) {
                for p in results.thumbnail_paths {
                    info!("Would delete {}", p.path().display());
                }
            }
            show!(
                "Found {thumbnail_count} thumbnail(s) to delete. Use '-v' for details, or '-f/--force' to delete them."
            );
        }
    } else {
        cached_delete(&results.thumbnail_paths);
    }

    Ok(thumbnail_count != 0)
}
