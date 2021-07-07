use anyhow::*;
use globset::{Candidate, GlobSet};
use log::*;
use png_pong::{chunk::Chunk, Decoder};
use std::fs::{remove_file, File};
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, io::BufReader, os::unix::prelude::OsStrExt};
use url::Url;
use walkdir::WalkDir;

pub mod cli;

#[derive(Debug)]
pub struct UnThumbnailer {
    pub recursive: bool,
    pub hidden: bool,
    cache_locs: [PathBuf; 2],
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

    pub fn delete(&self, paths: &[PathBuf], dry_run: bool) -> Result<(u32, u32)> {
        let mut nb_thumbs = 0;
        let mut nb_ignore_dirs = 0;

        let remove_fn = |t: &Thumbnail| {
            let p = t.original.to_string_lossy();
            if dry_run {
                info!("Would delete a thumbnail for {}", p);
                Ok(())
            } else {
                info!("Deleting a thumbnail for '{}'", p);

                remove_file(&t.thumbnail)
                    .with_context(|| format!("Failed to delete {}", t.thumbnail.to_string_lossy()))
            }
        };

        for path in paths.iter() {
            if path.is_file() {
                nb_thumbs += do_for_thumbnail(path, &self.cache_locs, remove_fn)?;
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
                    } else {
                        nb_thumbs += do_for_thumbnail(entry.path(), &self.cache_locs, remove_fn)?;
                    }
                }
            }
        }

        Ok((nb_thumbs, nb_ignore_dirs))
    }

    pub fn locate(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut thumb_path = Vec::new();

        let locate_fn = |t: &Thumbnail| {
            thumb_path.push(t.thumbnail.to_path_buf());
            Ok(())
        };

        do_for_thumbnail(path, &self.cache_locs, locate_fn)?;

        Ok(thumb_path)
    }

    pub fn cleanup(&self, force: bool, exclude: &GlobSet, include: &GlobSet) -> Result<u32> {
        let mut nb_thumbs = 0;
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
                nb_thumbs += match clean_thumbnail(entry.path(), force, &exclude, &include) {
                    Ok(nb) => nb,
                    Err(e) => {
                        if log_enabled!(log::Level::Trace) {
                            trace!("{} for {}", e, entry.path().to_string_lossy());
                        } else {
                            debug!("{} for {}", e, entry.path().to_string_lossy());
                        }
                        0
                    }
                };
            }
        }

        Ok(nb_thumbs)
    }
}

struct Thumbnail<'a> {
    thumbnail: &'a Path,
    original: &'a Path,
}

fn do_for_thumbnail<F>(path: &Path, locations: &[PathBuf; 2], mut f: F) -> Result<u32>
where
    F: FnMut(&Thumbnail) -> Result<()>,
{
    let mut nb_thumbs = 0;

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
            nb_thumbs += 1;
            let th = Thumbnail {
                thumbnail: &thumb,
                original: &path,
            };
            f(&th)?;
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

    Ok(nb_thumbs)
}

fn is_hidden_unix(str: &OsStr) -> bool {
    let c: char = str.as_bytes()[0].into();
    c == '.'
}

fn find_cache_locations() -> Result<[PathBuf; 2]> {
    let mut cache = dirs::cache_dir().ok_or_else(|| anyhow!("Could not find cache directory"))?;
    cache.push("thumbnails/");

    // TODO this ignores errors in iterating the subdirs
    let locations = [cache.join("normal"), cache.join("large")];
    if log_enabled!(log::Level::Debug) {
        debug!("Will look for thumbnails in the following directories:");
        for loc in &locations {
            debug!("{}", loc.to_string_lossy());
        }
    }

    Ok(locations)
}

fn clean_thumbnail(path: &Path, force: bool, exclude: &GlobSet, include: &GlobSet) -> Result<u32> {
    trace!("Processing {:?}", path);
    let mut nb_thumbs = 0;
    let origin = find_uri_for_thumbnail(path)?;

    let origin_url = Url::parse(&origin).map_err(|s| format_err!("{}", s))?;
    if origin_url.scheme() == "file" {
        let origin_path = origin_url.to_file_path().unwrap();
        let glob_candidate = Candidate::new(&origin_path);
        if !exclude.is_match_candidate(&glob_candidate)
            && include.is_match_candidate(&glob_candidate)
            && !origin_path.exists()
        {
            nb_thumbs += 1;
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
        }
    } else {
        trace!(
            "found a thumbnail origin URI with scheme {}, ignoring.",
            origin_url.scheme()
        );
    }

    Ok(nb_thumbs)
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
            Err(e) => panic!("Other Error: {:?}", e),
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
