use anyhow::*;
use env_logger::{Builder, Env};
use globset::{Glob, GlobSetBuilder};
use log::*;
use std::process::exit;
use structopt::StructOpt;
use thumbs::cli::{Command, ProgramOptions};
use thumbs::show;

const LOG_ENV_VAR: &str = "THUMBS_LOG";

fn main() {
    match run() {
        // Everything ok
        Ok(true) => exit(0),
        // Found nothing to do
        Ok(false) => exit(125),
        Err(e) => {
            let causes = e.chain().skip(1);
            if causes.len() != 0 {
                if log_enabled!(Level::Info) {
                    show!("Error: {}", e);
                    for cause in e.chain().skip(1) {
                        info!("cause: {}", cause);
                    }
                } else {
                    show!("Error: {}; rerun with '-v' for more information", e);
                }
            } else {
                show!("Error: {}", e);
            }
            exit(1)
        }
    }
}

fn run() -> Result<bool> {
    let args = ProgramOptions::from_args();

    let mut b = Builder::default();
    b.format_timestamp(None);
    b.filter_level(LevelFilter::Warn); // default filter lever
    b.parse_env(Env::from(LOG_ENV_VAR)); // override with env
                                         // override with CLI option
    if let Some(level) = args.log_level_with_default(2) {
        b.filter_level(level);
    };
    b.try_init()?;

    let un = thumbs::UnThumbnailer::new(args.recursive, args.all)?;
    match &args.cmd {
        Command::Cleanup { force, glob } => {
            let mut builder_exclude = GlobSetBuilder::new();
            let mut builder_include = GlobSetBuilder::new();
            let mut include_all = true;
            for g in glob {
                if g.starts_with('!') {
                    builder_exclude.add(Glob::new(g.strip_prefix('!').unwrap())?);
                } else {
                    include_all = false;
                    builder_include.add(Glob::new(g)?);
                }
            }
            if include_all {
                builder_include.add(Glob::new("**")?);
            }
            let set_exclude = builder_exclude.build()?;
            let set_include = builder_include.build()?;

            let nb_thumbs = un.cleanup(*force, &set_exclude, &set_include)?;

            if nb_thumbs == 0 {
                warn!("Found no thumbnails to cleanup. Rerun with '-vv' for detailed information.")
            } else if !force {
                show!(
                        "Found {} thumbnail(s) to delete. Use '-v' for details, or add '-f/--force' to delete them.",
                        nb_thumbs
                    );
            } else {
                show!("Deleted {} thumbnail(s).", nb_thumbs);
            }

            Ok(nb_thumbs != 0)
        }
        Command::Delete { dry_run, files } => {
            let results = un.delete(&files, *dry_run)?;
            if results.ignored_directories != 0 {
                warn!(
                    "Ignoring {} folder(s). Enable '-r/--recursive' to recurse into directories.",
                    results.ignored_directories
                )
            }
            if results.thumbnail_count == 0 {
                warn!("Found no thumbnails. Rerun with '-vv' for detailed information.")
            } else if *dry_run {
                show!(
                        "Found {} thumbnail(s) to delete. Use '-v' for details, or remove '-d/--dry-run' to delete them.",
                        results.thumbnail_count
                    );
            } else {
                show!("Deleted {} thumbnail(s).", results.thumbnail_count);
            }

            Ok(results.thumbnail_count != 0)
        }
        Command::Locate { file } => {
            let thumbs = un.locate(&file)?;

            for p in &thumbs {
                show!("{}", p.to_string_lossy());
            }

            Ok(!thumbs.is_empty())
        }
    }
}
