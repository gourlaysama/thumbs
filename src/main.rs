use anyhow::{Result, anyhow};
use clap::{CommandFactory, FromArgMatches};
use env_logger::{Builder, Env};
use globset::{Glob, GlobSetBuilder};
use log::*;
use std::process::{ExitCode, Termination};
use thumbs::cli::{Command, ProgramOptions};
use thumbs::{build, cleanup, locate, show};

use thumbs::delete;

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

    if args_matches.get_flag("help") {
        if args.cmd.is_none() {
            // HACK again, this only happens if -h/--help is set
            // use this instead of ArgAction::Help because the latter creates a conflict with subcommands somehow...
            let i = args_matches
                .index_of("help")
                .ok_or_else(|| anyhow!("should never happen: help set yet no version flag"))?;
            let mut command = ProgramOptions::command();
            if std::env::args().nth(i).unwrap_or_default() == "-h" {
                command.print_help()?;
            } else {
                command.print_long_help()?;
            }
            return Ok(true);
        } else if let Some(_cmd) = args.cmd {
            let mut command = ProgramOptions::command();
            let (name, sub_args) = args_matches.subcommand().unwrap();
            let hidx = sub_args
                .index_of("help")
                .ok_or_else(|| anyhow!("should never happen: help set yet no version flag"))?;
            let idx = std::env::args().position(|a| a == name).unwrap();
            println!("name:{name}, hidx:{hidx}, idx:{idx}");
            if std::env::args().nth(idx + hidx).unwrap_or_default() == "-h" {
                command.print_help()?;
            } else {
                command.print_long_help()?;
            }

            return Ok(true);
        };
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

    let cmd = args
        .cmd
        .expect("unexpected command, should have been caught by clap.");

    match cmd {
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

            cleanup::run(force, &set_exclude, &set_include)
        }
        Command::Delete {
            recursive,
            force,
            files,
            last_accessed,
            all,
        } => delete::run(files.as_ref(), force, last_accessed, recursive, all),
        Command::Locate { file } => locate::run(&file),
    }
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
