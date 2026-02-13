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

pub struct ThumbnailCache {
    cache_locations: Vec<PathBuf>,
}

// public methods
impl ThumbnailCache {
    pub fn init() -> Result<Self, io::Error> {
        Ok(ThumbnailCache {
            cache_locations: find_cache_locations()?,
        })
    }

    pub fn cache_locations(&self) -> impl Iterator<Item = &Path> {
        self.cache_locations.iter().map(PathBuf::as_path)
    }

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
                let thumb = Thumbnail::from_path(&thumb_path)?;
                match thumb.is_stale() {
                    Ok(true) => {
                        debug!("  Found      {:?}", thumb_path);
                    }
                    Ok(false) => thumbs.push(thumb),
                    Err(e) => {
                        trace!("Ignoring error '{e}' for {}", thumb_path.to_string_lossy())
                    }
                }
            }
        }

        Ok(thumbs.into_iter())
    }

    pub fn find_thumbnails_for_files_in(
        &self,
        files: &[PathBuf],
        last_accessed: Option<SystemTime>,
        recursive: bool,
        hidden: bool,
    ) -> TResult<SearchResults> {
        let mut thumbnails: Vec<Thumbnail> = Vec::new();
        let mut nb_ignore_dirs = 0;

        for path in files.iter() {
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
    let mut cache = dirs::cache_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "unable to locate XDG Cache directory",
        )
    })?;
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