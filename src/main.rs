use anyhow::*;
use env_logger::{Builder, Env};
use log::*;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::StructOpt;
use thumbs::cli::Command;
use thumbs::cli::ProgramOptions;
use url::Url;
use walkdir::WalkDir;

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

    match &args.cmd {
        #[cfg(any(feature = "cleanup", feature = "cleanup-magick7"))]
        Command::Cleanup { force, glob } => {
            let mut builder_exclude = GlobSetBuilder::new();
            let mut builder_include = GlobSetBuilder::new();
            let mut include_all = true;
            for g in glob {
                if g.starts_with('!') {
                    builder_exclude.add(Glob::new(&g[1..])?);
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
            cleanup(&args, *force, &set_exclude, &set_include)
        }
        _ => locate_or_delete(&args),
    }
}

fn locate_or_delete(args: &ProgramOptions) -> Result<bool> {
    let (files, dry_run) = match &args.cmd {
        Command::Delete { files, dry_run } => (files, *dry_run),
        Command::Locate { files } => (files, false),
        #[cfg(any(feature = "cleanup", feature = "cleanup-magick7"))]
        _ => panic!("Unreachable code; this is a bug."),
    };

    let locations = find_cache_location(true)?;
    let mut nb_thumbs = 0;

    for path in files.iter() {
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
        } else if path.is_dir() {
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

    
        if nb_thumbs == 0 {
            warn!("Found no thumbnails. Rerun with '-vv/--verbose 2' for detailed information.")
        } else if dry_run {
            show!(
                "Found {} thumbnail(s) to delete. Use '-v/--verbose' for details, or remove '-d/--dry-run' to delete them.",
                nb_thumbs
            );
        } else if let Command::Delete { .. } = args.cmd {
            show!("Deleted {} thumbnail(s).", nb_thumbs);
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

fn find_cache_location(include_fail: bool) -> Result<Vec<PathBuf>> {
    let mut cache = dirs::cache_dir().ok_or_else(|| anyhow!("Could not find cache directory"))?;
    cache.push("thumbnails/");

    // TODO this ignores errors in iterating the subdirs
    let mut locations: Vec<_> = if include_fail {
        cache
            .join("fail")
            .read_dir()?
            .flatten()
            .map(|e| e.path())
            .collect()
    } else {
        Vec::new()
    };
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

fn handle_file(path: &Path, args: &ProgramOptions, locations: &[PathBuf]) -> Result<u32> {
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
                        
                            info!("Would delete a thumbnail for {}", path.to_string_lossy());
                        
                    } else {
                        
                            info!("Deleting a thumbnail for '{}'", path.to_string_lossy());
                        
                        remove_file(&thumb).with_context(|| {
                            format!("Failed to delete {}", thumb.to_string_lossy())
                        })?;
                    }
                }
                Command::Locate { .. } => {
                    //show!(thumb.to_string_lossy());
                }
                #[cfg(any(feature = "cleanup", feature = "cleanup-magick7"))]
                Command::Cleanup { .. } => {}
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

#[cfg(feature = "cleanup-magick7")]
use magick_rust::{magick_wand_genesis, magick_wand_terminus, MagickWand};
#[cfg(feature = "cleanup")]
use magick_rust_6::{magick_wand_genesis, magick_wand_terminus, MagickWand};

#[cfg(any(feature = "cleanup", feature = "cleanup-magick7"))]
fn cleanup(
    args: &ProgramOptions,
    force: bool,
    exclude: &GlobSet,
    include: &GlobSet,
) -> Result<bool> {
    magick_wand_genesis();

    let locations = find_cache_location(false)?;
    let mut nb_thumbs = 0;
    for location in locations {
        for entry in WalkDir::new(location)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| args.all || !file_is_hidden(e))
            .filter_map(|e| e.ok())
            .filter(|e| {
                !e.file_type().is_dir() && e.path().extension().map_or(false, |p| p == "png")
            })
        {
            nb_thumbs += match clean_thumbnail(entry.path(), &args, force, &exclude, &include) {
                Ok(nb) => nb,
                Err(e) => {
                    if log_enabled!(log::Level::Trace) {
                        trace!(
                            "{} for {}",
                            e.display_causes_and_backtrace(),
                            entry.path().to_string_lossy()
                        );
                    } else {
                        debug!(
                            "{} for {}",
                            e.display_causes_without_backtrace(),
                            entry.path().to_string_lossy()
                        );
                    }
                    0
                }
            };
        }
    }

    magick_wand_terminus();

    if !args.quiet {
        if nb_thumbs == 0 {
            warn!("Found no thumbnails to cleanup. Rerun with '-vv/--verbose 2' for detailed information.")
        } else if !force {
            show!(
                "Found {} thumbnail(s) to delete. Use '-v/--verbose' for details, or add '-f/--force' to delete them.",
                nb_thumbs
            );
        } else {
            show!("Deleted {} thumbnail(s).", nb_thumbs);
        }
    }

    Ok(!force || nb_thumbs != 0)
}

#[cfg(any(feature = "cleanup", feature = "cleanup-magick7"))]
fn clean_thumbnail(
    path: &Path,
    args: &ProgramOptions,
    force: bool,
    exclude: &GlobSet,
    include: &GlobSet,
) -> Result<u32> {
    trace!("Processing {:?}", path);
    let mut nb_thumbs = 0;
    let origin = {
        let wand = MagickWand::new();
        let path_str = path.to_string_lossy();
        wand.read_image(&path_str)
            .map_err(|s| format_err!("{}", s))?;
        wand.get_image_property("Thumb::URI")
            .map_err(|s| format_err!("{}", s))?
    };

    let origin_url = Url::parse(&origin).map_err(|s| format_err!("{}", s))?;
    if origin_url.scheme() == "file" {
        let origin_path = origin_url.to_file_path().unwrap();
        let glob_candidate = Candidate::new(&origin_path);
        if !exclude.is_match_candidate(&glob_candidate)
            && include.is_match_candidate(&glob_candidate)
            && !origin_path.exists()
        {
            nb_thumbs += 1;
            if !force {
                if !args.quiet {
                    info!(
                        "Would delete a thumbnail for {}",
                        origin_path.to_string_lossy()
                    );
                }
            } else {
                if !args.quiet {
                    info!(
                        "Deleting a thumbnail for '{}'",
                        origin_path.to_string_lossy()
                    );
                }
                remove_file(&path).io_write_context(path)?;
            }
        }
    }

    Ok(nb_thumbs)
}

#[macro_export]
macro_rules! show {
    ($level:ident, $($a:tt)*) => {
        if log_enabled!(log::Level::$level) {
            println!($($a)*);
        }
    };
    ($($a:tt)*) => {
        if log_enabled!(log::Level::Error) {
            println!($($a)*);
        }
    }
}
