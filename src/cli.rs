use anyhow::{bail, Result};
use clap::ValueHint;
use log::LevelFilter;
use std::{path::PathBuf, time::SystemTime};

#[derive(Debug, clap::Parser)]
#[clap(
    about = "Utility to find and delete generated thumbnails.",
    setting = clap::AppSettings::NoAutoVersion,
    mut_arg("help", |h| h.help_heading("INFO")),
    mut_arg("version", |h| h.help_heading("INFO")),
)]
pub struct ProgramOptions {
    /// Pass for more log output.
    #[clap(
        long,
        short,
        global = true,
        parse(from_occurrences),
        help_heading = "FLAGS"
    )]
    verbose: i8,

    /// Pass for less log output.
    #[clap(
        long,
        short,
        global = true,
        parse(from_occurrences),
        conflicts_with = "verbose",
        help_heading = "FLAGS"
    )]
    quiet: i8,

    #[clap(short, long, help_heading = "FLAGS", global = true)]
    /// Recurse through directories
    pub recursive: bool,

    #[clap(short, long, help_heading = "FLAGS", global = true)]
    /// Include hidden files and directories
    pub all: bool,

    #[clap(subcommand)]
    pub cmd: Option<Command>,
}

impl ProgramOptions {
    pub fn log_level_with_default(&self, default: i8) -> Option<LevelFilter> {
        let level = default + self.verbose - self.quiet;
        let new_level = match level {
            i8::MIN..=0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5..=i8::MAX => LevelFilter::Trace,
        };

        if level != default {
            Some(new_level)
        } else {
            None
        }
    }
}

#[derive(Debug, clap::Parser)]
pub enum Command {
    /// Delete the thumbnails for the given files
    #[clap(setting = clap::AppSettings::NoAutoVersion)]
    Delete {
        #[clap(short, long, help_heading = "FLAGS")]
        /// Do not prompt and actually delete thumbnails
        force: bool,

        #[clap(parse(from_os_str), value_hint(ValueHint::FilePath), value_name = "FILE")]
        /// Files whose thumbnails to delete
        files: Vec<PathBuf>,

        /// Only delete thumbnails for files that haven't been accessed since the given time.
        ///
        /// Can be either a RFC3339-like timestamp (`2020-01-01 11:10:00`) or a free-form
        /// duration like `1year 15days 1week 2min` or `1h 6s 2ms`.
        #[clap(short, long, parse(try_from_str = parse_last_accessed))]
        last_accessed: Option<SystemTime>,
    },
    /// Print the path of thumbnails for the given files
    #[clap(setting = clap::AppSettings::NoAutoVersion)]
    Locate {
        #[clap(parse(from_os_str), value_hint(ValueHint::FilePath), value_name = "FILE")]
        /// File whose thumbnails are to be found
        file: PathBuf,
    },
    /// Find thumbnails for files that no longer exist
    #[clap(setting = clap::AppSettings::NoAutoVersion)]
    Cleanup {
        #[clap(short, long, help_heading = "FLAGS")]
        /// Actually delete thumbnails
        force: bool,

        #[clap(short, long, value_name = "GLOB")]
        /// Include or exclude files and directories that match the given globs. Can be used
        /// multiple times. Globbing rules match .gitignore globs. Precede a glob with a !
        /// to exclude it.
        glob: Vec<String>,
    },
}

fn parse_last_accessed(s: &str) -> Result<SystemTime> {
    if let Ok(t) = humantime::parse_rfc3339_weak(s) {
        return Ok(t);
    }

    if let Ok(d) = humantime::parse_duration(s) {
        return Ok(SystemTime::now() - d);
    }

    bail!(
        "Cannot parse '{}' as either a RFC3339-like timestamp or a free-form duration",
        s
    );
}
