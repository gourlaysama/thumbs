use anyhow::Result;
use cli::Command;
use globset::{Glob, GlobSetBuilder};
use log::*;
use std::{io::IsTerminal, sync::LazyLock};

use thumbs_rs::Thumbnail;

pub mod cleanup;
pub mod cli;
pub mod delete;
pub mod interactive;
pub mod locate;
pub mod utils;

pub(crate) fn cached_delete(thumbnails: &[Thumbnail]) {
    if log::log_enabled!(Level::Debug) {
        for p in thumbnails {
            let uri = p.source_uri();
            if uri.scheme() == "file" {
                if let Ok(p) = uri.to_file_path() {
                    debug!("Deleting thumbnail for {}", p.display());
                } else {
                    debug!("Deleting thumbnail for {}", uri);
                }
            } else {
                debug!("Deleting thumbnail for {}", uri);
            }

            if let Err(e) = p.delete() {
                warn!("{e}");
            }
        }
    } else {
        for p in thumbnails {
            if let Err(e) = p.delete() {
                warn!("{e}");
            }
        }
    }

    show!("Deleted {} thumbnail(s).", thumbnails.len());
}

pub static STDOUT_IS_TERMINAL: LazyLock<bool> = LazyLock::new(|| std::io::stdout().is_terminal());

pub fn run(cmd: Command) -> Result<bool> {
    let changed = match cmd {
        Command::Cleanup { force, glob } => {
            let mut builder_exclude = GlobSetBuilder::new();
            let mut builder_include = GlobSetBuilder::new();
            let mut include_all = true;
            for g in glob {
                if g.starts_with('!') {
                    builder_exclude.add(Glob::new(g.strip_prefix('!').unwrap())?);
                } else {
                    include_all = false;
                    builder_include.add(Glob::new(&g)?);
                }
            }
            if include_all {
                builder_include.add(Glob::new("**")?);
            }
            let set_exclude = builder_exclude.build()?;
            let set_include = builder_include.build()?;

            cleanup::run(force, &set_exclude, &set_include)?
        }
        Command::Delete {
            recursive,
            force,
            files,
            last_accessed,
            all,
        } => {
            delete::run(
                files.iter().map(|p| p.as_path()),
                force,
                last_accessed,
                recursive,
                all,
            )?
        }
        Command::Locate { file } => locate::run(&file)?,
    };

    if !changed {
        show!("Nothing to do.")
    }

    Ok(changed)
}

#[macro_export]
macro_rules! show {
    ($($a:tt)*) => {
        {
            if *crate::STDOUT_IS_TERMINAL {
                println!($($a)*);
            } else {
                log::info!($($a)*);
            }
        }
    }
}
