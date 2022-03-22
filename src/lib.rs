use anyhow::{anyhow, format_err, Context, Result};
use globset::{Candidate, GlobSet};
use log::*;
use png_pong::{chunk::Chunk, Decoder};
use std::fs::{remove_file, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{ffi::OsStr, io::BufReader, os::unix::prelude::OsStrExt};
use url::Url;
use walkdir::{DirEntry, WalkDir};

pub mod cli;

#[derive(Debug)]
pub struct UnThumbnailer {
    pub recursive: bool,
    pub hidden: bool,
    cache_locs: Vec<PathBuf>,
}

impl UnThumbnailer {
    pub fn new(recursive: bool, hidden: bool) -> Result<Self> {
        let cache_locs = find_cache_locations()?;
        Ok(Self {
            recursive,
            hidden,
            cache_locs,
        })
    }

    /// Delete thumbnails for the files at `paths`, possibly recursing in directories
    /// if enabled. `dry_run` only reports results but doesn't actually delete
    /// anything.
    pub fn delete(
        &self,
        paths: &[PathBuf],
        dry_run: bool,
        last_accessed: Option<SystemTime>,
    ) -> Result<DeleteResults> {
        let mut thumbs = Vec::new();
        let mut nb_ignore_dirs = 0;

        let mode = if dry_run { Mode::DryRun } else { Mode::Delete };

        for path in paths.iter() {
            if path.is_file() {
                do_for_thumbnail(path, &self.cache_locs, &mut thumbs, mode)?;
            } else {
                let mut walk = WalkDir::new(path).min_depth(1);
                if !self.recursive {
                    walk = walk.max_depth(1);
                }
                for entry in walk
                    .into_iter()
                    .filter_entry(|e| self.hidden || !is_hidden_unix(e.file_name()))
                    .filter_map(|e| e.ok())
                {
                    trace!("entry: {:?}", entry);
                    if entry.file_type().is_dir() {
                        if !self.recursive {
                            nb_ignore_dirs += 1;
                        }
                    } else if let Some(last_accessed) = last_accessed {
                        fn entry_was_accessed_since(e: &DirEntry, t: SystemTime) -> Result<bool> {
                            let acc_t = e.metadata()?.accessed()?;

                            Ok(acc_t >= t)
                        }

                        match entry_was_accessed_since(&entry, last_accessed) {
                            Ok(false) => {
                                do_for_thumbnail(
                                    entry.path(),
                                    &self.cache_locs,
                                    &mut thumbs,
                                    mode,
                                )?;
                            }
                            Ok(true) => {}
                            Err(e) => {
                                debug!(
                                    "Failed to find accesstime of {}",
                                    entry.path().to_string_lossy()
                                );
                                trace!("Failed with {}", e);
                            }
                        }
                    } else {
                        do_for_thumbnail(entry.path(), &self.cache_locs, &mut thumbs, mode)?;
                    }
                }
            }
        }

        Ok(DeleteResults {
            thumbnail_paths: thumbs,
            ignored_directories: nb_ignore_dirs,
        })
    }

    /// Locate the thumbnails for a path.
    ///
    /// The path has to point to a file. Multiple results can be returned because
    /// multiple thumbnails with different sizes can be returned for the same
    /// source.
    pub fn locate(&self, path: &Path) -> Result<Vec<Thumbnail>> {
        let mut thumbs = Vec::new();

        do_for_thumbnail(path, &self.cache_locs, &mut thumbs, Mode::Locate)?;

        Ok(thumbs)
    }

    /// Delete thumbnails for files that don't exist.
    ///
    /// The `exclude` and `include` globs constrain the search to thumbnails whose original
    /// files match them.
    ///
    /// Returns the number of matching thumbnails.
    pub fn cleanup(
        &self,
        force: bool,
        exclude: &GlobSet,
        include: &GlobSet,
    ) -> Result<Vec<Thumbnail>> {
        let mut thumbs = Vec::new();
        for location in &self.cache_locs {
            for entry in WalkDir::new(location)
                .min_depth(1)
                .into_iter()
                .filter_entry(|e| self.hidden || !is_hidden_unix(e.file_name()))
                .filter_map(|e| e.ok())
                .filter(|e| {
                    !e.file_type().is_dir() && e.path().extension().map_or(false, |p| p == "png")
                })
            {
                match clean_thumbnail(entry.path(), force, exclude, include, &mut thumbs) {
                    Ok(_) => {}
                    Err(e) => {
                        if log_enabled!(log::Level::Trace) {
                            trace!("{} for {}", e, entry.path().to_string_lossy());
                        } else {
                            debug!("{} for {}", e, entry.path().to_string_lossy());
                        }
                    }
                };
            }
        }

        Ok(thumbs)
    }
}

#[derive(Debug)]
pub struct DeleteResults {
    pub thumbnail_paths: Vec<Thumbnail>,
    pub ignored_directories: u32,
}

#[derive(Debug, Clone)]
pub struct Thumbnail {
    pub thumbnail: PathBuf,
    pub file: PathBuf,
}

#[derive(Copy, Clone)]
enum Mode {
    Locate,
    DryRun,
    Delete,
}

fn do_for_thumbnail(
    path: &Path,
    locations: &[PathBuf],
    acc_paths: &mut Vec<Thumbnail>,
    mode: Mode,
) -> Result<()> {
    // TODO is canonicalize too much? (it resolves symlinks)
    let url = if !path.is_absolute() {
        Url::from_file_path(&path.canonicalize()?)
    } else {
        Url::from_file_path(&path)
    }
    .map_err(|_| format_err!("Non absolute path: {:?}", &path))?;
    trace!("Url: {:?}", url);

    let digest = md5::compute(url.as_str().as_bytes());

    debug!("Processing {:?} ({:x})", path, digest);

    let mut thumb_seen = false;

    for location in locations.iter() {
        let mut thumb = location.clone();
        thumb.push(format!("{:x}", digest));
        thumb.set_extension("png");
        if thumb.exists() {
            debug!("  Found      {:?}", thumb);
            thumb_seen = true;
            match mode {
                Mode::Locate => {}
                Mode::DryRun => {
                    info!("Would delete a thumbnail for {}", path.to_string_lossy());
                }
                Mode::Delete => {
                    info!("Deleting a thumbnail for '{}'", path.to_string_lossy());

                    remove_file(&thumb)
                        .with_context(|| format!("Failed to delete {}", thumb.to_string_lossy()))?;
                }
            }
            let th = Thumbnail {
                thumbnail: thumb,
                file: path.to_path_buf(),
            };
            acc_paths.push(th);
        } else {
            debug!("  Not found  {:?}", thumb);
        }
    }

    if !thumb_seen {
        debug!(
            "Could not find a thumbnail for '{}'",
            path.to_string_lossy()
        );
    }

    Ok(())
}

fn is_hidden_unix(str: &OsStr) -> bool {
    let c: char = str.as_bytes()[0].into();
    c == '.'
}

fn find_cache_locations() -> Result<Vec<PathBuf>> {
    let mut cache = dirs::cache_dir().ok_or_else(|| anyhow!("Could not find cache directory"))?;
    cache.push("thumbnails/");

    // TODO this ignores errors in iterating the subdirs
    let init_locations = [
        cache.join("normal"),
        cache.join("large"),
        cache.join("fail"),
    ];
    let mut locations = Vec::new();
    for loc in init_locations {
        let walk = WalkDir::new(&loc);

        for entry in walk
            .into_iter()
            .filter_entry(|e| e.file_type().is_dir())
            .filter_map(|e| e.ok())
        {
            trace!("entry: {:?}", entry);
            locations.push(entry.into_path());
        }
    }

    if log_enabled!(log::Level::Debug) {
        debug!("Will look for thumbnails in the following directories:");
        for loc in &locations {
            debug!("{}", loc.to_string_lossy());
        }
    }

    Ok(locations)
}

fn clean_thumbnail(
    path: &Path,
    force: bool,
    exclude: &GlobSet,
    include: &GlobSet,
    acc_paths: &mut Vec<Thumbnail>,
) -> Result<()> {
    trace!("Processing {:?}", path);
    let origin = find_uri_for_thumbnail(path)?;

    let origin_url = Url::parse(&origin).map_err(|s| format_err!("{}", s))?;
    if origin_url.scheme() == "file" {
        let origin_path = origin_url.to_file_path().unwrap();
        let glob_candidate = Candidate::new(&origin_path);
        if !exclude.is_match_candidate(&glob_candidate)
            && include.is_match_candidate(&glob_candidate)
            && !origin_path.exists()
        {
            if !force {
                if log_enabled!(log::Level::Info) {
                    info!(
                        "Would delete a thumbnail for {}",
                        origin_path.to_string_lossy()
                    );
                }
            } else {
                if log_enabled!(log::Level::Info) {
                    info!(
                        "Deleting a thumbnail for '{}'",
                        origin_path.to_string_lossy()
                    );
                }
                remove_file(&path)
                    .with_context(|| format!("failed to delete file {}", path.to_string_lossy()))?;
            }
            let th = Thumbnail {
                thumbnail: path.to_path_buf(),
                file: origin_path,
            };
            acc_paths.push(th);
        }
    } else {
        trace!(
            "found a thumbnail origin URI with scheme {}, ignoring.",
            origin_url.scheme()
        );
    }

    Ok(())
}

fn find_uri_for_thumbnail(path: &Path) -> Result<String> {
    let reader = BufReader::new(File::open(path)?);
    for chunk in Decoder::new(reader)?.into_chunks() {
        match chunk {
            Ok(c) => match c {
                Chunk::CompressedText(text) => {
                    if text.key == "Thumb::URI" {
                        return Ok(text.val);
                    }
                }
                Chunk::Text(text) => {
                    if text.key == "Thumb::URI" {
                        return Ok(text.val);
                    }
                }
                _ => (),
            },
            Err(e) => {
                trace!("ignored error: {}", e);
            }
        }
    }

    Err(anyhow!("failed to find origin path"))
}

#[macro_export]
macro_rules! show {
    ($level:ident, $($a:tt)*) => {
        if log_enabled!(log::Level::$level) {
            println!($($a)*);
        }
    };
    ($($a:tt)*) => {
        if log_enabled!(log::Level::Error) {
            println!($($a)*);
        }
    }
}
