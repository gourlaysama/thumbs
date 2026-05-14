use clap::CommandFactory;
use clap_complete::{Shell, generate_to};

include!("../../src/cli.rs");

fn main() -> Result<(), std::io::Error> {
    let matches = clap::Command::new("xtask")
        .subcommand(clap::Command::new("gen-completions"))
        .get_matches();

    if let Some(("gen-completions", _)) = matches.subcommand() {
        let mut outdir = std::env::current_dir()?;
        outdir.push("extra/complete");
        std::fs::create_dir_all(&outdir)?;

        println!("Generating completions in {}", outdir.display());

        let mut app = ProgramOptions::command();

        generate_to(Shell::Bash, &mut app, "thumbs", &outdir)?;

        generate_to(Shell::Zsh, &mut app, "thumbs", &outdir)?;

        generate_to(Shell::Fish, &mut app, "thumbs", &outdir)?;
    }

    Ok(())
}
