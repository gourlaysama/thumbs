use anyhow::*;
use log::LevelFilter;
use std::{path::PathBuf, time::SystemTime};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Utility to find and delete generated thumbnails.")]
pub struct ProgramOptions {
    /// Pass for more log output.
    #[structopt(long, short, global = true, parse(from_occurrences))]
    verbose: i8,

    /// Pass for less log output.
    #[structopt(
        long,
        short,
        global = true,
        parse(from_occurrences),
        conflicts_with = "verbose"
    )]
    quiet: i8,

    #[structopt(short, long, global = true)]
    /// Recurse through directories
    pub recursive: bool,

    #[structopt(short, long, global = true)]
    /// Include hidden files and directories
    pub all: bool,

    #[structopt(subcommand)]
    pub cmd: Command,
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

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Delete the thumbnails for the given files
    Delete {
        #[structopt(short, long)]
        /// Do not actually delete anything
        dry_run: bool,

        #[structopt(parse(from_os_str))]
        /// Files whose thumbnails to delete
        files: Vec<PathBuf>,

        /// Only delete thumbnails for files that haven't been accessed since the given time.
        ///
        /// Can be either a RFC3339-like timestamp (`2020-01-01 11:10:00`) or a free-form
        /// duration like `1year 15days 1week 2min` or `1h 6s 2ms`.
        #[structopt(short, long, parse(try_from_str = parse_last_accessed))]
        last_accessed: Option<SystemTime>,
    },
    /// Print the path of thumbnails for the given files
    Locate {
        #[structopt(parse(from_os_str))]
        /// File whose thumbnails are to be found
        file: PathBuf,
    },
    /// Find thumbnails for files that no longer exist
    Cleanup {
        #[structopt(short, long)]
        /// Actually delete thumbnails
        force: bool,

        #[structopt(short, long)]
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
