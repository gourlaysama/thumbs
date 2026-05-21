use std::fs;

use crate::*;
use png_pong::chunk::{Chunk, ImageData, ImageEnd, ImageHeader, Text};

pub type R = anyhow::Result<()>;

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

pub fn make_empty_thumbnail(dir: &Path, source: &Path) -> anyhow::Result<PathBuf> {
    make_thumbnail(dir, source, None, None)
}

pub fn make_thumbnail(
    dir: &Path,
    source: &Path,
    a_time: Option<u64>,
    size: Option<u64>,
) -> anyhow::Result<PathBuf> {
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
    if let Some(a_time) = a_time {
        encoder.encode(&mut Chunk::Text(Text {
            key: "Thumb::MTime".to_string(),
            val: format!("{}", a_time),
        }))?;
    }
    if let Some(size) = size {
        encoder.encode(&mut Chunk::Text(Text {
            key: "Thumb::Size".to_string(),
            val: format!("{}", size),
        }))?;
    }
    encoder.encode(&mut Chunk::ImageEnd(ImageEnd))?;

    fs::write(&result, &data)?;

    Ok(result)
}
