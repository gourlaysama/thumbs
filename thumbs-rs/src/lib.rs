/*!
 * **Thumbnail management library for systems respecting [Freedesktop's Thumbnail Management Standard](https://specifications.freedesktop.org/thumbnail/latest/index.html)**
 * 
 * # Example
 * 
 * ```
 * use std::env
 * use std::path::Path
 * use thumbs_rs::ThumbnailCache
 * 
 * fn main() -> Result<()> {
 *      let path = env::args.next().unwrap();
 *      let cache = ThumbnailCache::init()?;
 * 
 *      for thumbnail in cache.find_thumbnails_for_file(&path)? {
 *          println!("found: {:?}", thumbnail.path())
 *          if thumbnail.is_stale()? {
 *              println!("thumbnail is not up to date, deleting...")
 *              thumbnail.delete()?;
 *          }
 *      }
 * }
 * ```
 */

use etcetera::{BaseStrategy, base_strategy::Xdg};
use globset::{Candidate, GlobSet};
use log::*;
use percent_encoding::{AsciiSet, percent_encode};
use std::{
    ffi::OsStr,
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    result::Result,
    str,
    time::SystemTime,
};
use thiserror::Error;
use walkdir::WalkDir;

pub use crate::thumbnail::Thumbnail;

pub mod thumbnail;

type TResult<T> = Result<T, ThumbnailError>;

/// The thumbnail cache.
/// 
/// This is the entry point for all operations on thumbnails.
pub struct ThumbnailCache {
    cache_locations: Vec<PathBuf>,
}

// public methods
impl ThumbnailCache {
    /// Find the thumbnail cache for the current user.
    ///
    /// # Errors
    ///
    /// This function will return an error if the home directory is unknown or if the cache location
    /// is inaccessible.
    pub fn init() -> Result<Self, io::Error> {
        Ok(ThumbnailCache {
            cache_locations: find_cache_locations()?,
        })
    }

    /// Returns the cache locations of this [`ThumbnailCache`].
    /// 
    /// This includes the directories used to cache the placeholder thumbnails used when thumbnail
    /// generation failed.
    pub fn cache_locations(&self) -> impl Iterator<Item = &Path> {
        self.cache_locations.iter().map(PathBuf::as_path)
    }

    /// Finds all the thumbnails for a file at the given path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file path is invalid, does not exists of cannot
    /// be resolved.
    pub fn find_thumbnails_for_file(
        &self,
        path: &Path,
    ) -> TResult<impl Iterator<Item = Thumbnail>> {
        let encoded_path = make_encoded_uri(path)?;
        let digest = md5::compute(encoded_path.as_bytes());
        debug!("Processing {path:?} ({digest:x})");

        let mut thumbs = Vec::new();

        for location in self.cache_locations.iter() {
            let mut thumb_path = location.clone();
            thumb_path.push(format!("{digest:x}"));
            thumb_path.set_extension("png");
            if thumb_path.exists() {
                match Thumbnail::from_path(&thumb_path) {
                    Ok(t) => thumbs.push(t),
                    Err(e) => {
                        debug!(
                            "Thumbnail {} exists but is invalid: {e}",
                            thumb_path.display()
                        )
                    }
                }
            }
        }

        Ok(thumbs.into_iter())
    }

    /// Finds all the thumbnails for the given files and for files within the given directories.
    ///
    /// If an input path is a file, this appends the results of [`find_thumbnail_for_file`] to the output.
    ///
    /// If an input path is a directory, it appends the results of [`find_thumbnail_for_file`] to the output
    /// for all files within. Hidden files are included if `hidden` is set. Directories further down will
    /// only be searched if `recursive` is set.
    ///
    /// When a timestamp is provided in `last_accessed`, only thumbnails for files with an older access
    /// time will be returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the paths are invalid, do not exists or cannot
    /// be resolved.
    pub fn find_thumbnails_for_files_in(
        &self,
        paths: &[PathBuf],
        last_accessed: Option<SystemTime>,
        recursive: bool,
        hidden: bool,
    ) -> TResult<SearchResults> {
        let mut thumbnails: Vec<Thumbnail> = Vec::new();
        let mut nb_ignore_dirs = 0;

        for path in paths.iter() {
            if path.is_file() {
                thumbnails.extend(self.find_thumbnails_for_file(path.as_path())?);
                continue;
            }

            let mut walk = WalkDir::new(path).min_depth(1);
            if !recursive {
                walk = walk.max_depth(1);
            }

            for entry in walk
                .into_iter()
                .filter_entry(|e| hidden || !is_hidden_unix(e.file_name()))
                .filter_map(|e| e.ok())
            {
                trace!("entry: {entry:?}");
                if entry.file_type().is_dir() {
                    if !recursive {
                        nb_ignore_dirs += 1;
                        continue;
                    }
                } else if let Some(last_accessed) = last_accessed {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(e) => {
                            debug!(
                                "Failed to find metadata of {}",
                                entry.path().to_string_lossy()
                            );
                            trace!("Failed with: {e}");
                            continue;
                        }
                    };

                    let acc_t = metadata
                        .accessed()
                        .expect("never happens on supported platforms");

                    if acc_t >= last_accessed {
                        continue;
                    }
                };

                thumbnails.extend(self.find_thumbnails_for_file(entry.path())?);
            }
        }

        Ok(SearchResults {
            thumbnail_paths: thumbnails,
            ignored_directories: nb_ignore_dirs,
        })
    }

    /// Finds all thumbnails in the cache whose files are missing.
    ///
    /// Search through the entire thumbnail cache for thumbnails whose corresponding file is missing.
    /// The `exclude` and `include` globsets are checked in that order against the thumbnail's
    /// corresponding file.
    /// 
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn find_thumbnails_for_missing_files(
        &self,
        exclude: &GlobSet,
        include: &GlobSet,
    ) -> TResult<impl Iterator<Item = Thumbnail>> {
        let mut thumbs = Vec::new();
        for location in &self.cache_locations {
            for entry in WalkDir::new(location)
                .min_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    !e.file_type().is_dir() && e.path().extension().map_or(false, |p| p == "png")
                })
            {
                let path = entry.path();
                trace!("Processing {path:?}");

                let thumb = Thumbnail::from_path(path)?;
                let origin_path = thumb
                    .source_uri()
                    .to_file_path()
                    .expect("currently required, should never happen");
                let glob_candidate = Candidate::new(&origin_path);
                if !exclude.is_match_candidate(&glob_candidate)
                    && include.is_match_candidate(&glob_candidate)
                    && !origin_path.exists()
                {
                    thumbs.push(thumb);
                }
            }
        }

        Ok(thumbs.into_iter())
    }
}

#[derive(Debug, Error)]
#[error("error processing thumbnail at {path}")]
pub struct ThumbnailError {
    path: PathBuf,
    source: ThumbnailErrorSource,
}

impl ThumbnailError {
    fn url_error(path: &Path, e: url::ParseError) -> Self {
        ThumbnailError {
            path: path.into(),
            source: ThumbnailErrorSource::InvalidSourceURI(Box::new(e)),
        }
    }

    fn utf8_error(path: &Path, e: str::Utf8Error) -> Self {
        ThumbnailError {
            path: path.into(),
            source: ThumbnailErrorSource::InvalidSourceURI(Box::new(e)),
        }
    }

    fn from(path: &Path, source: ThumbnailErrorSource) -> Self {
        ThumbnailError {
            path: path.into(),
            source,
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ThumbnailErrorSource {
    #[error("invalid png file")]
    Png(#[from] png_pong::decode::Error),

    #[error("the URI for the source file is missing from thumbnail metadata")]
    MissingMetadata,

    #[error("invalid souce file URI")]
    InvalidSourceURI(Box<dyn std::error::Error + Send + Sync>),

    #[error("invalid URI scheme '{0}', expected 'file'")]
    UnsupportedScheme(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl ThumbnailErrorSource {
    fn with_path(self, path: PathBuf) -> ThumbnailError {
        ThumbnailError { path, source: self }
    }
}

impl From<url::ParseError> for ThumbnailErrorSource {
    fn from(value: url::ParseError) -> Self {
        ThumbnailErrorSource::InvalidSourceURI(Box::new(value))
    }
}

impl From<str::Utf8Error> for ThumbnailErrorSource {
    fn from(value: str::Utf8Error) -> Self {
        ThumbnailErrorSource::InvalidSourceURI(Box::new(value))
    }
}

#[derive(Debug)]
pub struct SearchResults {
    pub thumbnail_paths: Vec<Thumbnail>,
    pub ignored_directories: u32,
}

const CUSTOM_ENCODING_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'*')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'>')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'{')
    .add(b'|')
    .add(b'}');

fn make_encoded_uri(path: &Path) -> TResult<String> {
    let inner = if !path.is_absolute() {
        path.canonicalize()
            .map_err(|e| ThumbnailError::from(path, e.into()))?
            .into_os_string()
    } else {
        path.into()
    };

    let mut url = String::new();
    url.push_str("file://");
    url.extend(percent_encode(inner.as_bytes(), &CUSTOM_ENCODING_SET));
    trace!("Encoded Url: {url:?}");

    Ok(url)
}

fn find_cache_locations() -> Result<Vec<PathBuf>, io::Error> {
    let mut cache = Xdg::new()
        .or_else(|e| Err(io::Error::new(io::ErrorKind::NotFound, e)))?
        .cache_dir();
    cache.push("thumbnails/");

    // TODO this ignores errors in iterating the subdirs
    let init_locations = [
        cache.join("normal"),
        cache.join("large"),
        cache.join("x-large"),
        cache.join("xx-large"),
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
            trace!("entry: {entry:?}");
            locations.push(entry.into_path());
        }
    }

    if log_enabled!(log::Level::Debug) {
        debug!("Will look for thumbnails in the following directories:");
        for loc in &locations {
            debug!("  {}", loc.to_string_lossy());
        }
    }

    Ok(locations)
}

fn is_hidden_unix(str: &OsStr) -> bool {
    let c: char = str.as_bytes()[0].into();
    c == '.'
}
