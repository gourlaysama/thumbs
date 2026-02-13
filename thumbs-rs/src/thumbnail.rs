use log::*;
use percent_encoding::percent_decode_str;
use png_pong::{Decoder, chunk::Chunk};
use std::{
    fs::File,
    io::BufReader,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};
use url::Url;

use crate::{TResult, ThumbnailError, ThumbnailErrorSource};

#[derive(Debug)]
pub struct Thumbnail {
    inner: PathBuf,
    source_uri: Url,
    m_time: Option<u64>,
    size: Option<u64>,
}

impl Thumbnail {
    pub fn from_path(path: &Path) -> TResult<Thumbnail> {
        // TODO is canonicalize too much? (it resolves symlinks)
        let inner = if !path.is_absolute() {
            path.canonicalize()
                .map_err(|e| ThumbnailError::from(path, e.into()))?
        } else {
            path.into()
        };
        let mut encoded_uri: Option<String> = None;
        let mut m_time = None;
        let mut size = None;

        let reader =
            BufReader::new(File::open(&inner).map_err(|e| ThumbnailError::from(path, e.into()))?);
        for chunk in Decoder::new(reader)
            .map_err(|e| ThumbnailError::from(path, e.into()))?
            .into_chunks()
        {
            match chunk {
                Ok(c) => match c {
                    Chunk::CompressedText(text) if text.key == "Thumb::URI" => {
                        encoded_uri = Some(text.val);
                    }
                    Chunk::Text(text) if text.key == "Thumb::URI" => {
                        encoded_uri = Some(text.val);
                    }
                    Chunk::CompressedText(text) if text.key == "Thumb::MTime" => {
                        match text.val.parse() {
                            Ok(v) => m_time = Some(v),
                            Err(e) => {
                                trace!("ignored error: {e}");
                            }
                        }
                    }
                    Chunk::Text(text) if text.key == "Thumb::MTime" => match text.val.parse() {
                        Ok(v) => m_time = Some(v),
                        Err(e) => {
                            trace!("ignored error: {e}");
                        }
                    },
                    Chunk::CompressedText(text) if text.key == "Thumb::Size" => {
                        match text.val.parse() {
                            Ok(v) => size = Some(v),
                            Err(e) => {
                                trace!("ignored error: {e}");
                            }
                        }
                    }
                    Chunk::Text(text) if text.key == "Thumb::Size" => match text.val.parse() {
                        Ok(v) => size = Some(v),
                        Err(e) => {
                            trace!("ignored error: {e}");
                        }
                    },
                    _ => (),
                },
                Err(e) => {
                    trace!("ignored error: {e}");
                }
            };
        }

        let source_decoded = if let Some(u) = encoded_uri {
            percent_decode_str(&u)
                .decode_utf8()
                .map_err(|e| ThumbnailError::utf8_error(path.into(), e))?
                .into_owned()
        } else {
            return Err(ThumbnailErrorSource::MissingMetadata.with_path(path.into()));
        };

        let source_uri =
            Url::parse(&source_decoded).map_err(|e| ThumbnailError::url_error(path.into(), e))?;

        // TODO support more
        if source_uri.scheme() != "file" {
            return Err(
                ThumbnailErrorSource::UnsupportedScheme(source_uri.scheme().into())
                    .with_path(path.into()),
            );
        }

        Ok(Thumbnail {
            inner,
            source_uri,
            m_time,
            size,
        })
    }

    pub fn is_stale(&self) -> TResult<bool> {
        let path = self.source_uri.to_file_path().unwrap();
        if !path.exists() {
            return Ok(true);
        }

        let metadata = path
            .metadata()
            .map_err(|e| ThumbnailError::from(self.path(), e.into()))?;
        if let Some(m_time) = self.m_time {
            if m_time
                != metadata
                    .modified()
                    .expect("never happens on supported platforms")
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("UNIX_EPOCH can never be earlier than anything")
                    .as_secs()
            {
                return Ok(true);
            }
        }

        if let Some(size) = self.size {
            if size != metadata.size() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn delete(&self) -> TResult<()> {
        Ok(std::fs::remove_file(&self.inner)
            .map_err(|e| ThumbnailErrorSource::Io(e).with_path(self.inner.clone()))?)
    }

    pub fn source_uri(&self) -> &Url {
        &self.source_uri
    }

    pub fn m_time(&self) -> Option<u64> {
        self.m_time
    }

    pub fn size(&self) -> Option<u64> {
        self.size
    }

    pub fn path(&self) -> &Path {
        &self.inner
    }

    pub fn exists(&self) -> bool {
        self.inner.exists()
    }
}
