use std::env::var;
use std::process;

fn main() {
    assert!(
        var("CARGO_FEATURE_CLEANUP").is_err() || var("CARGO_FEATURE_CLEANUP_MAGICK7").is_err(),
        "the 'cleanup' and 'cleanup-magick7' features cannot be set at the same time: they do the
same thing with bindings for ImageMagick 6 and 7, respectively."
    );

    let is_tag = process::Command::new("git")
        .args(&["describe", "--exact-match", "--tags"])
        .stderr(process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !is_tag {
        if let Ok(out) = process::Command::new("git")
            .args(&["rev-parse", "--short=7", "HEAD"])
            .output()
        {
            let hash = String::from_utf8_lossy(&out.stdout);

            println!("cargo:rustc-env=THUMB_GIT_HASH={}", hash.trim());
        }
    }

    built::write_built_file().expect("Failed to acquire build-time information.");
}
