use std::{fs, path::Path};

use png_pong::chunk::{Chunk, ImageData, ImageEnd, ImageHeader, Text};

use super::*;

type R = anyhow::Result<()>;

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

fn make_empty_thumbnail(dir: &Path, source: &Path) -> anyhow::Result<PathBuf> {
    let mut encoded_path = String::new();
    encoded_path.push_str("file://");
    encoded_path.extend(percent_encode(
        source.as_os_str().as_bytes(),
        CUSTOM_ENCODING_SET,
    ));

    let digest = md5::compute(encoded_path.as_bytes());
    let name = format!("{digest:x}.png");
    let result = dir.join(&name);
    println!(
        "creating {} for {}",
        result.to_string_lossy(),
        source.to_string_lossy()
    );
    let mut data = PNG_SIGNATURE.to_vec();
    let mut encoder = png_pong::Encoder::new(&mut data).into_chunk_enc();

    encoder.encode(&mut Chunk::ImageHeader(ImageHeader {
        width: 256,
        height: 256,
        color_type: png_pong::chunk::ColorType::Rgb,
        bit_depth: 8,
        interlace: false,
    }))?;

    encoder.encode(&mut Chunk::ImageData(ImageData { data: Vec::new() }))?;
    encoder.encode(&mut Chunk::Text(Text {
        key: "Thumb::URI".to_string(),
        val: encoded_path,
    }))?;
    encoder.encode(&mut Chunk::ImageEnd(ImageEnd))?;

    fs::write(&result, &data)?;

    Ok(result)
}

#[test]
fn read_thumbnail() -> R {
    let temp = assert_fs::TempDir::new()?;
    let source = Path::new("/tmp/thumbs/test.jpg");

    let res = make_empty_thumbnail(temp.path(), source)?;
    let thumb = Thumbnail::from_path(&res)?;

    assert_eq!(thumb.path(), res);
    assert_eq!(thumb.source_uri().to_file_path().unwrap(), source);

    Ok(())
}

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
