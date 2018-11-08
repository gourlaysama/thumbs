use common_failures::prelude::*;
use failure::format_err;
use log::*;
use std::fs::remove_file;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "unthumnailer", about = "Deletes cached thumnails for files.")]
struct Cli {
    #[structopt(short = "d", long = "dry-run")]
    /// Do not actually delete anything
    dry_run: bool,

    #[structopt(short, long, parse(from_occurrences))]
    /// Verbosity
    verbose: usize,

    #[structopt(short, long)]
    /// Quiets all output
    quiet: bool,

    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>,
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

    if args.files.is_empty() {
        Cli::clap().print_help()?;
        return Ok(true);
    }

    let mut nb_thumbs = 0;

    for path in args.files {
        // TODO is canonicalize too much? (it resolves symlinks)
        let path = if !path.is_absolute() {
            path.canonicalize()?
        } else {
            path
        };
        let url = Url::from_file_path(&path)
            .map_err(|_| format_err!("Non absolute path: {:?}", &path))?;
        trace!("Url: {:?}", url);
        let digest = md5::compute(url.as_str().as_bytes());

        debug!("Processing {:?} ({:x})", path, digest);

        let mut loc =
            dirs::home_dir().ok_or_else(|| format_err!("Could not find home directory"))?;
        loc.push(".cache/thumbnails/");

        let mut thumb_seen = false;

        for tpe in ["normal", "large", "fail/gnome-thumbnail-factory"].iter() {
            let mut thumb = PathBuf::from(&loc.as_path());
            thumb.push(tpe);
            thumb.push(format!("{:x}", digest));
            thumb.set_extension("png");
            if thumb.exists() {
                debug!("  Found      {:?}", thumb);
                thumb_seen = true;
                nb_thumbs += 1;
                if args.dry_run {
                    if !args.quiet {
                        info!("Would delete a thumbnail for {}", path.to_string_lossy());
                    }
                } else {
                    if !args.quiet {
                        info!("Deleting a thumbnail for '{}'", path.to_string_lossy());
                    }
                    remove_file(&thumb).io_write_context(&thumb)?;
                }
            } else {
                debug!("  Not found  {:?}", thumb);
            }
        }

        if !thumb_seen {
            info!(
                "Could not find a thumbnail for '{}'",
                path.to_string_lossy()
            );
        }
    }

    if !args.quiet {
        if nb_thumbs == 0 {
            warn!("Found no thumbnail to delete. Rerun with '-vv' for detailed information.")
        } else if args.dry_run {
            println!(
                "Found {} thumbnail(s) to delete: add '-v' for details, or remove '--dry-run' to delete.",
                nb_thumbs
            );
        } else {
            println!("Deleted {} thumbnail(s).", nb_thumbs);
        }
    }

    if args.dry_run {
        Ok(true)
    } else {
        Ok(nb_thumbs != 0)
    }
}
