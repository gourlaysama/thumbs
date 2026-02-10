use anyhow::{Result, anyhow};
use clap::{CommandFactory, FromArgMatches};
use env_logger::{Builder, Env};
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::*;
use std::io::{IsTerminal, Write, stdin};
use std::path::PathBuf;
use std::process::{ExitCode, Termination};
use std::time::SystemTime;
use thumbs::cli::{Command, ProgramOptions};
use thumbs::{Thumbnail, UnThumbnailer, build, show};

const LOG_ENV_VAR: &str = "THUMBS_LOG";

#[repr(u8)]
pub enum ThumbsResult {
    OK = 0,
    Error = 1,
    NothingToDo = 125,
}

impl Termination for ThumbsResult {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

fn main() -> ThumbsResult {
    match run() {
        // Everything ok
        Ok(true) => ThumbsResult::OK,
        // Found nothing to do
        Ok(false) => ThumbsResult::NothingToDo,
        Err(e) => {
            let causes = e.chain().skip(1);
            if causes.len() != 0 {
                if log_enabled!(Level::Info) {
                    show!("Error: {e}");
                    for cause in causes {
                        info!("cause: {cause}");
                    }
                } else {
                    show!("Error: {e}; rerun with '-v' for more information");
                }
            } else {
                show!("Error: {e}");
            }
            ThumbsResult::Error
        }
    }
}

fn run() -> Result<bool> {
    let args_matches = ProgramOptions::command().get_matches();
    let args = ProgramOptions::from_arg_matches(&args_matches)?;

    if args_matches.get_flag("version") {
        // HACK to disambiguate short/long invocations for the same cli option;
        // there has to be a better way of doing this...
        let i = args_matches
            .index_of("version")
            .ok_or_else(|| anyhow!("should never happen: version set yet no version flag"))?;
        if std::env::args().nth(i).unwrap_or_default() == "-V" {
            print_version(false);
        } else {
            print_version(true);
        }
        return Ok(true);
    }

    let mut b = Builder::default();
    b.format_timestamp(None);
    b.filter_level(LevelFilter::Warn); // default filter lever
    b.parse_env(Env::from(LOG_ENV_VAR)); // override with env
    // override with CLI option
    if let Some(level) = args.log_level_with_default(2) {
        b.filter_level(level);
    };
    b.try_init()?;

    let cmd = args.cmd.expect("unexpected command, should have been caught by clap.");

    let un = thumbs::UnThumbnailer::new()?;
    match cmd {
        Command::Cleanup { force, glob, all } => {
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

            do_cleanup(&un, force, &set_exclude, &set_include, all)
        }
        Command::Delete {
            recursive,
            force,
            files,
            last_accessed,
            all,
        } => do_delete(&un, files.as_ref(), force, last_accessed, recursive, all),
        Command::Locate { file } => {
            let thumbs = un.locate(file.as_ref())?;

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
    hidden: bool,
) -> Result<bool> {
    let thumbs = un.cleanup(force, set_exclude, set_include, hidden)?;
    let nb_thumbs = thumbs.len();
    if nb_thumbs == 0 {
        warn!("Found no thumbnails to cleanup.")
    } else if !force {
        if stdin().is_terminal() {
            return user_prompt(&thumbs, || cached_delete(&thumbs));
        } else {
            show!(
                "Found {nb_thumbs} thumbnail(s) to delete. Use '-v' for details, or '-f/--force' to delete them."
            );
        }
    } else {
        show!("Deleted {nb_thumbs} thumbnail(s).");
    }

    Ok(nb_thumbs != 0)
}

fn do_delete(
    un: &UnThumbnailer,
    files: &[PathBuf],
    force: bool,
    last_accessed: Option<SystemTime>,
    recursive: bool,
    hidden: bool,
) -> Result<bool> {
    let results = un.delete(files, !force, last_accessed, recursive, hidden)?;
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
        if stdin().is_terminal() {
            return user_prompt(&results.thumbnail_paths, || {
                cached_delete(&results.thumbnail_paths)
            });
        } else {
            show!(
                "Found {thumbnail_count} thumbnail(s) to delete. Use '-v' for details, or '-f/--force' to delete them."
            );
        }
    } else {
        show!("Deleted {thumbnail_count} thumbnail(s).");
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
        trace!("read user input: {confirm:?}");

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

fn print_version(long: bool) {
    println!(
        "{} {}{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        get_version_suffix(),
    );

    if long {
        println!("\n{}\n{}", build::RUST_VERSION, build::BUILD_TARGET);

        if shadow_rs::is_debug() {
            println!("\n+debug");
        }
    }
}

fn get_version_suffix() -> String {
    let mut suffix = String::new();

    if build::TAG != "" {
        // this is a tagged release
        return suffix;
    }

    if build::COMMIT_HASH == "" {
        // this is a tarball or equivalent
        return suffix;
    }

    if let Some(last_tag) = build::LAST_TAG.strip_prefix('v') {
        if last_tag == env!("CARGO_PKG_VERSION") {
            // this is a later commit on top of some previous version
            suffix = format!("+git.{}.{}", build::COMMITS_SINCE_TAG, build::SHORT_COMMIT);
            if !build::GIT_CLEAN {
                suffix.push_str(".dirty");
            }
        }
    }

    suffix
}
