use std::process;

fn main() {
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
