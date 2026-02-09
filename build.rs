use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use shadow_rs::ShadowBuilder;
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Err(Error::new(std::io::ErrorKind::Other, "no $OUT_DIR!")),
        Some(outdir) => outdir,
    };
    let mut app = ProgramOptions::command();

    generate_to(Shell::Bash, &mut app, "thumbs", &outdir)?;

    generate_to(Shell::Zsh, &mut app, "thumbs", &outdir)?;

    generate_to(Shell::Fish, &mut app, "thumbs", outdir)?;

    ShadowBuilder::builder().build().unwrap();

    Ok(())
}
