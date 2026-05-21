use std::path::Path;

use super::*;
use common::*;

mod common;
mod thumbnail;

#[test]
fn find_thumbnail() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source = Path::new("/tmp/thumbs/test.jpg");

    let result = make_empty_thumbnail(temp.path(), source)?;

    let cache = ThumbnailCache::init_with(vec![temp.to_path_buf()])?;

    let mut res = cache.find_thumbnails_for_file(source)?;
    let next = res.next();
    assert!(next.is_some());

    let thumb = next.unwrap();
    assert_eq!(thumb.path(), result);
    assert_eq!(thumb.source_uri().to_file_path().unwrap(), source);

    temp.close()?;

    Ok(())
}
