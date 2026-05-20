use std::{os::unix::fs::MetadataExt, path::Path, time::SystemTime};

use assert_fs::fixture::{FileTouch, PathChild};

use crate::Thumbnail;

use super::common::*;

#[test]
fn thumbnail_from_path() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source = Path::new("/tmp/thumbs/test.jpg");

    let res = make_empty_thumbnail(temp.path(), source)?;
    let thumb = Thumbnail::from_path(&res)?;

    assert_eq!(thumb.path(), res);
    assert_eq!(thumb.source_uri().to_file_path().unwrap(), source);

    temp.close()?;

    Ok(())
}

#[test]
fn deleted_file_thumb_is_stale() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source_temp = assert_fs::TempDir::new()?;
    let source = &source_temp.child("foo.jpg");

    let res = make_empty_thumbnail(temp.path(), source)?;
    let thumb = Thumbnail::from_path(&res)?;

    source.touch()?;
    assert!(!thumb.is_stale()?);
    std::fs::remove_file(source.path())?;
    assert!(thumb.is_stale()?);

    temp.close()?;
    source_temp.close()?;

    Ok(())
}

#[test]
fn different_mtime_file_thumb_is_stale() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source_temp = assert_fs::TempDir::new()?;
    let source = &source_temp.child("foo.jpg");

    source.touch()?;

    // thumbnail made a long time ago
    let res = make_thumbnail(temp.path(), source, Some(10), None)?;
    let thumb = Thumbnail::from_path(&res)?;
    assert!(thumb.is_stale()?);

    // thumbnail made in the future
    let m_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs() + 300;
    let res = make_thumbnail(temp.path(), source, Some(m_time), None)?;
    let thumb = Thumbnail::from_path(&res)?;
    assert!(thumb.is_stale()?);

    // same m_time
    let m_time = source.metadata()?.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    let res = make_thumbnail(temp.path(), source, Some(m_time), None)?;
    let thumb = Thumbnail::from_path(&res)?;
    assert!(!thumb.is_stale()?);

    temp.close()?;
    source_temp.close()?;

    Ok(())
}

#[test]
fn different_size_file_thumb_is_stale() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source_temp = assert_fs::TempDir::new()?;
    let source = &source_temp.child("foo.jpg");

    source.touch()?;

    // thumbnail for smaller file
    let res = make_thumbnail(temp.path(), source, None, Some(10))?;
    let thumb = Thumbnail::from_path(&res)?;
    assert!(thumb.is_stale()?);

    // thumbnail for bigger file
    let size = source.metadata()?.size() + 1000;
    let res = make_thumbnail(temp.path(), source, None, Some(size))?;
    let thumb = Thumbnail::from_path(&res)?;
    assert!(thumb.is_stale()?);

    // same size
    let size = source.metadata()?.size();
    let res = make_thumbnail(temp.path(), source, None, Some(size))?;
    let thumb = Thumbnail::from_path(&res)?;
    assert!(!thumb.is_stale()?);

    temp.close()?;
    source_temp.close()?;

    Ok(())
}