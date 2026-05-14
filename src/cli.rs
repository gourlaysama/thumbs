use anyhow::Result;
use clap::{ArgAction, ValueHint, builder::styling};
use clap_stdin::MaybeStdin;
use log::LevelFilter;
use std::{path::PathBuf, time::SystemTime};

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

#[cfg(not_build_rs)]
const FULL_LONG_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/full_long_version.txt"));

/// Utility to find and delete generated thumbnails.
/// 
/// Foo bar baz.
#[derive(Debug, clap::Parser)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    propagate_version = true,
    styles = STYLES,
    disable_help_subcommand = true,
)]
#[cfg_attr(not_build_rs, command(
    version = env!("FULL_VERSION"),
    long_version = FULL_LONG_VERSION,
))]
pub struct ProgramOptions {
    /// Pass for more log output.
    #[clap(
        long,
        short,
        global = true,
        action = ArgAction::Count,
        help_heading = "Global Flags"
    )]
    verbose: u8,

    /// Pass for less log output.
    #[clap(
        long,
        short,
        global = true,
        action = ArgAction::Count,
        conflicts_with = "verbose",
        help_heading = "Global Flags"
    )]
    quiet: u8,

    /// Print help.
    #[clap(
        long,
        short,
        action = ArgAction::Help,
        exclusive = true,
        global = true,
        help_heading = "Global Flags"
    )]
    help: bool,

    /// Print version.
    #[clap(
        long,
        short = 'V',
        action = ArgAction::Version,
        exclusive = true,
        global = true,
        help_heading = "Global Flags"
    )]
    version: bool,

    #[clap(subcommand)]
    pub cmd: Option<Command>,
}

impl ProgramOptions {
    pub fn log_level_with_default(&self, default: i16) -> LevelFilter {
        let level = default + self.verbose as i16 - self.quiet as i16;
        match level {
            i16::MIN..=0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5..=i16::MAX => LevelFilter::Trace,
        }
    }
}

#[derive(Debug, clap::Parser)]
pub enum Command {
    /// Delete the thumbnails for the given files
    Delete {
        #[clap(short, long, help_heading = "Flags")]
        /// Recurse through directories
        recursive: bool,

        #[clap(short, long, help_heading = "Flags")]
        /// Do not prompt and actually delete thumbnails
        force: bool,

        #[clap(short, long, help_heading = "Flags", global = true)]
        /// Include hidden files and directories
        all: bool,

        #[clap(required = true, value_parser = clap::value_parser!(MaybeStdin<PathBuf>), value_hint(ValueHint::FilePath), value_name = "FILE")]
        /// File whose thumbnails to delete, or `-` for reading a file path from standard input.
        /// 
        /// Multiple paths can be given, but only a single one can use `-` for standard input. Whatever
        /// can be read over standard input will be treated as a single path; use `xargs` or similar to
        /// programatically giving multiple paths.
        files: Vec<MaybeStdin<PathBuf>>,

        /// Only delete thumbnails for files that haven't been accessed in the given time.
        ///
        /// Can be either a RFC3339-like timestamp (`2020-01-01 11:10:00`) or a free-form
        /// duration like `1year 15days 1week 2min` or `1h 6s 2ms`.
        #[clap(short, long, value_parser = parse_last_accessed)]
        last_accessed: Option<SystemTime>,
    },
    /// Print the path of thumbnails for the given files
    Locate {
        #[clap(value_parser = clap::value_parser!(MaybeStdin<PathBuf>), value_hint(ValueHint::FilePath), value_name = "FILE")]
        /// File whose thumbnails are to be found, or `-` to read a file path from standard input.
        file: MaybeStdin<PathBuf>,
    },
    /// Find thumbnails for files that no longer exist and optionally delete them
    Cleanup {
        #[clap(short, long, help_heading = "Flags")]
        /// Do not prompt and actually delete thumbnails.
        force: bool,

        #[clap(value_name = "GLOB")]
        /// Include or exclude files and directories that match the given globs. Can be used
        /// multiple times. Globbing rules match `.gitignore` globs. Precede a glob with a `!`
        /// to exclude it.
        /// 
        /// If no including glob is given, all files are included first before excluding globs are
        /// considered.
        glob: Vec<String>,
    },
    /// Show information about a file's thumbnails or the thumbnail cache
    Info {
        /// A file to provide information about, or `-` to read a file path from standard input.
        /// 
        /// If no file is given, global information about the thumbnail cache itself is returned.
        file: Option<MaybeStdin<PathBuf>>
    }
}

#[cfg(not_build_rs)]
fn parse_last_accessed(s: &str) -> Result<SystemTime> {
    if let Ok(t) = humantime::parse_rfc3339_weak(s) {
        return Ok(t);
    }

    if let Ok(d) = humantime::parse_duration(s) {
        return Ok(SystemTime::now() - d);
    }

    anyhow::bail!("Cannot parse '{s}' as either a RFC3339-like timestamp or a free-form duration");
}

// this is to avoid a build-dependency on humantime, this is not actually called at build time
#[cfg(not(not_build_rs))]
fn parse_last_accessed(_s: &str) -> Result<SystemTime> {
    Ok(SystemTime::UNIX_EPOCH)
}
