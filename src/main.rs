use common_failures::prelude::*;
use failure::format_err;
use lazy_static::lazy_static;
use log::*;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use url::Url;
use walkdir::WalkDir;

lazy_static! {
    static ref VERSION: String = {
        let hash = match option_env!("THUMB_GIT_HASH") {
            None => String::new(),
            Some(hash) => format!("-pre-{}", hash),
        };

        format!("{}{}", structopt::clap::crate_version!(), hash)
    };
}

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Utility to find and delete generated thumbnails.",
    rename_all = "kebab",
    raw(
        global_settings = "
        &[AppSettings::ColoredHelp,
          AppSettings::ArgRequiredElseHelp,
          AppSettings::VersionlessSubcommands,
          AppSettings::InferSubcommands]",
        version = "VERSION.as_str()"
    )
)]
struct Cli {
    #[structopt(short, long, parse(from_occurrences), raw(global = "true"))]
    /// Verbosity
    verbose: usize,

    #[structopt(short, long, raw(global = "true"))]
    /// Quiets all output
    quiet: bool,

    #[structopt(short, long, raw(global = "true"))]
    /// Recurse through directories
    recursive: bool,

    #[structopt(short, long, raw(global = "true"))]
    /// Include hidden files and directories
    all: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab_case")]
enum Command {
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
}

fn main() {
    match run() {
        // Everything ok
        Ok(true) => exit(0),
        // Found nothing to delete
        Ok(false) => exit(125),
        Err(e) => {
            // We can't log the error if it's the logger that failed
            if e.downcast_ref::<log::SetLoggerError>().is_some() {
                eprintln!("{}", e.display_causes_without_backtrace());
            } else {
                debug!("{}", e.display_causes_without_backtrace());
                error!("{}", e);
            }
            exit(1)
        }
    }
}

fn run() -> Result<bool> {
    let args = Cli::from_args();
    stderrlog::new().verbosity(args.verbose + 1).init()?;

    let (files, dry_run) = match &args.cmd {
        Command::Delete { files, dry_run } => (files, *dry_run),
        Command::Locate { files } => (files, false),
    };

    let locations = find_cache_location()?;
    let mut nb_thumbs = 0;

    for path in files {
        if args.recursive {
            for entry in WalkDir::new(path)
                .min_depth(1)
                .into_iter()
                .filter_entry(|e| args.all || !file_is_hidden(e))
                .filter_map(|e| e.ok())
                .filter(|e| !e.file_type().is_dir())
            {
                nb_thumbs += handle_file(entry.path(), &args, &locations)?;
            }
        } else if path.is_dir() && !args.quiet {
            warn!(
                "Ignoring directory {}. Use '-r/--recursive' to recurse into directories.",
                path.to_string_lossy()
            );
        } else if args.all
            || !path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        {
            nb_thumbs += handle_file(path, &args, &locations)?;
        }
    }

    if !args.quiet {
        if nb_thumbs == 0 {
            warn!("Found no thumbnails. Rerun with '-vv/--verbose 2' for detailed information.")
        } else if dry_run {
            println!(
                "Found {} thumbnail(s) to delete. Use '-v/--verbose' for details, or remove '-d/--dry-run' to delete them.",
                nb_thumbs
            );
        } else if let Command::Delete { .. } = args.cmd {
            println!("Deleted {} thumbnail(s).", nb_thumbs);
        }
    }

    if dry_run {
        Ok(true)
    } else {
        Ok(nb_thumbs != 0)
    }
}

fn file_is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn find_cache_location() -> Result<Vec<PathBuf>> {
    let mut cache =
        dirs::cache_dir().ok_or_else(|| format_err!("Could not find cache directory"))?;
    cache.push("thumbnails/");
    // TODO this ignores errors in iterating the subdirs
    let mut locations: Vec<_> = cache
        .join("fail")
        .read_dir()?
        .flat_map(|d| d)
        .map(|e| e.path())
        .collect();
    locations.push(cache.join("normal"));
    locations.push(cache.join("large"));
    if log_enabled!(log::Level::Debug) {
        debug!("Will look for thumbnails in the following directories:");
        for loc in &locations {
            debug!("{}", loc.to_string_lossy());
        }
    }

    Ok(locations)
}

fn handle_file(path: &Path, args: &Cli, locations: &[PathBuf]) -> Result<u32> {
    let mut nb_thumbs = 0;

    // TODO is canonicalize too much? (it resolves symlinks)
    let url = if !path.is_absolute() {
        Url::from_file_path(&path.canonicalize()?)
    } else {
        Url::from_file_path(&path)
    }
    .map_err(|_| format_err!("Non absolute path: {:?}", &path))?;
    trace!("Url: {:?}", url);

    let digest = md5::compute(url.as_str().as_bytes());

    debug!("Processing {:?} ({:x})", path, digest);

    let mut thumb_seen = false;

    for location in locations.iter() {
        let mut thumb = location.clone();
        thumb.push(format!("{:x}", digest));
        thumb.set_extension("png");
        if thumb.exists() {
            debug!("  Found      {:?}", thumb);
            thumb_seen = true;
            nb_thumbs += 1;
            match args.cmd {
                Command::Delete { dry_run, .. } => {
                    if dry_run {
                        if !args.quiet {
                            info!("Would delete a thumbnail for {}", path.to_string_lossy());
                        }
                    } else {
                        if !args.quiet {
                            info!("Deleting a thumbnail for '{}'", path.to_string_lossy());
                        }
                        remove_file(&thumb).io_write_context(thumb)?;
                    }
                }
                Command::Locate { .. } => {
                    println!("{}", thumb.to_string_lossy());
                }
            };
        } else {
            debug!("  Not found  {:?}", thumb);
        }
    }

    if !thumb_seen {
        debug!(
            "Could not find a thumbnail for '{}'",
            path.to_string_lossy()
        );
    }

    Ok(nb_thumbs)
}
