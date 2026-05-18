use build_info_build::VersionControl;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    build_info()
}

fn build_info() -> Result<(), Error> {
    let info = build_info_build::build_script()
        .collect_dependencies(build_info_build::DependencyDepth::Depth(1))
        .build();

    let mut full_version = info.crate_info.version.to_string();
    if let Some(VersionControl::Git(g)) = info.version_control {
        let exact_tag = g.tags.iter().any(|t| t.ends_with(&full_version));
        if exact_tag {
            // this is an exact tag release
            println!("cargo::rustc-env=FULL_VERSION={full_version}");
        }

        full_version.push_str("+git.");
        full_version.push_str(&g.commit_short_id);
        if g.dirty {
            full_version.push_str(".dirty");
        }

        if !exact_tag {
            println!("cargo::rustc-env=FULL_VERSION={full_version}");
        }
    } else if let Ok(build_id) = std::env::var("BUILD_ID") {
        println!("cargo::rustc-env=FULL_VERSION={full_version}");
        full_version.push('+');
        full_version.push_str(&build_id);
    } else {
        println!("cargo::rustc-env=FULL_VERSION={full_version}");
    }

    let dep = info
        .crate_info
        .dependencies
        .iter()
        .find(|c| c.name == "thumbs-rs")
        .expect("missing thumbs-rs?");
    full_version.push_str("\n\n");
    full_version.push_str(&dep.name);
    full_version.push(' ');
    full_version.push_str(&dep.version.to_string());

    full_version.push('\n');
    full_version.push_str(&info.compiler.to_string());
    full_version.push('\n');
    full_version.push_str(&info.target.triple);

    if info.profile != "release" {
        full_version.push_str("\n\n+");
        full_version.push_str(&info.profile);
    }

    let mut out = std::env::var("OUT_DIR").unwrap();
    out.push_str("/full_long_version.txt");
    std::fs::write(out, full_version)?;

    println!("cargo::rustc-check-cfg=cfg(not_build_rs)");
    println!("cargo::rustc-cfg=not_build_rs");

    Ok(())
}
