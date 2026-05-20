use std::path::Path;

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
fn thumbnail_is_stale() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source_temp = assert_fs::TempDir::new()?;
    let source = &source_temp.child("foo.jpg");

    let res = make_empty_thumbnail(temp.path(), source)?;
    let thumb = Thumbnail::from_path(&res)?;

    assert!(thumb.is_stale()?);
    source.touch()?;
    assert!(!thumb.is_stale()?);
    
    temp.close()?;
    source_temp.close()?;

    Ok(())
}