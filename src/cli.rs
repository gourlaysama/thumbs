use std::path::PathBuf;

use log::LevelFilter;
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
#[structopt(rename_all = "kebab")]
pub enum Command {
    /// Delete the thumbnails for the given files
    Delete {
        #[structopt(short, long)]
        /// Do not actually delete anything
        dry_run: bool,

        #[structopt(parse(from_os_str))]
        /// Files whose thumbnails to delete
        files: Vec<PathBuf>,
    },
    /// Print the path of thumbnails for the given files
    Locate {
        #[structopt(parse(from_os_str))]
        /// Files whose thumbnails to find
        files: Vec<PathBuf>,
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
