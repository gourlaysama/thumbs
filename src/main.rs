use anyhow::*;
use env_logger::{Builder, Env};
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::*;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::time::SystemTime;
use structopt::StructOpt;
use thumbs::cli::{Command, ProgramOptions};
use thumbs::{show, Thumbnail, UnThumbnailer};

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

            do_cleanup(&un, *force, &set_exclude, &set_include)
        }
        Command::Delete {
            force,
            files,
            last_accessed,
        } => do_delete(&un, files, *force, *last_accessed),
        Command::Locate { file } => {
            let thumbs = un.locate(&file)?;

            for p in &thumbs {
                show!("{}", p.thumbnail.to_string_lossy());
            }

            Ok(!thumbs.is_empty())
        }
    }
}

fn do_cleanup(
    un: &UnThumbnailer,
    force: bool,
    set_exclude: &GlobSet,
    set_include: &GlobSet,
) -> Result<bool> {
    let thumbs = un.cleanup(force, &set_exclude, &set_include)?;
    let nb_thumbs = thumbs.len();
    if nb_thumbs == 0 {
        warn!("Found no thumbnails to cleanup.")
    } else if !force {
        if atty::is(atty::Stream::Stdout) {
            return user_prompt(&thumbs, || cached_delete(&thumbs));
        } else {
            show!(
                "Found {} thumbnail(s) to delete. Use '-v' for details, or '-f/--force' to delete them.",
                nb_thumbs
            );
        }
    } else {
        show!("Deleted {} thumbnail(s).", nb_thumbs);
    }

    Ok(nb_thumbs != 0)
}

fn do_delete(
    un: &UnThumbnailer,
    files: &[PathBuf],
    force: bool,
    last_accessed: Option<SystemTime>,
) -> Result<bool> {
    let results = un.delete(files, !force, last_accessed)?;
    let thumbnail_count = results.thumbnail_paths.len();

    if results.ignored_directories != 0 {
        warn!(
            "Ignoring {} folder(s). Enable '-r/--recursive' to recurse into directories.",
            results.ignored_directories
        )
    }
    if thumbnail_count == 0 {
        warn!("Found no thumbnails. Rerun with '-vv' for detailed information.")
    } else if !force {
        if atty::is(atty::Stream::Stdout) {
            return user_prompt(&results.thumbnail_paths, || {
                cached_delete(&results.thumbnail_paths)
            });
        } else {
            show!(
                "Found {} thumbnail(s) to delete. Use '-v' for details, or '-f/--force' to delete them.",
                thumbnail_count
            );
        }
    } else {
        show!("Deleted {} thumbnail(s).", thumbnail_count);
    }

    Ok(thumbnail_count != 0)
}

fn user_prompt<F>(thumbnails: &[Thumbnail], on_yes: F) -> Result<bool>
where
    F: Fn() -> Result<()>,
{
    loop {
        {
            let out = std::io::stdout();
            let mut out = out.lock();
            write!(
                out,
                "Found {} thumbnail(s) to delete.\nDelete them? y(es) / N(o) / d(etails)> ",
                thumbnails.len()
            )?;
            out.flush()?;
        }

        let mut confirm = String::with_capacity(1);
        std::io::stdin().read_line(&mut confirm)?;
        trace!("read user input: {:?}", confirm);

        if confirm.eq_ignore_ascii_case("y\n") {
            on_yes()?;
            return Ok(!thumbnails.is_empty());
        } else if confirm.eq_ignore_ascii_case("d\n") {
            let out = std::io::stdout();
            let mut out = out.lock();
            writeln!(out, "Found thumbnails for:")?;
            for p in thumbnails {
                writeln!(out, "{}", p.file.to_string_lossy())?;
            }
            out.flush()?;
        } else {
            return Ok(!thumbnails.is_empty());
        }
    }
}

fn cached_delete(thumbnails: &[Thumbnail]) -> Result<()> {
    for p in thumbnails {
        std::fs::remove_file(&p.thumbnail)?;
    }

    show!("Deleted {} thumbnail(s).", thumbnails.len());
    Ok(())
}
