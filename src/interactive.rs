use anyhow::Result;
use log::*;
use std::io::Write;

use thumbs_rs::Thumbnail;

pub(crate) fn user_prompt<F>(thumbnails: &[Thumbnail], on_yes: F) -> Result<bool>
where
    F: Fn() -> (),
{
    loop {
        {
            let out = std::io::stdout();
            let mut out = out.lock();
            write!(
                out,
                "Found {} thumbnail(s) to delete.\nDelete them? y(es) / N(o) / d(etails)> ",
                thumbnails.len()
            )?;
            out.flush()?;
        }

        let mut confirm = String::with_capacity(1);
        std::io::stdin().read_line(&mut confirm)?;
        trace!("read user input: {confirm:?}");

        if confirm.eq_ignore_ascii_case("y\n") {
            on_yes();
            return Ok(!thumbnails.is_empty());
        } else if confirm.eq_ignore_ascii_case("d\n") {
            let out = std::io::stdout();
            let mut out = out.lock();
            writeln!(out, "Found thumbnails for:")?;
            for p in thumbnails {
                if let Ok(f) = p.source_uri().to_file_path() {
                    writeln!(out, "{}", f.display())?;
                }
            }
            out.flush()?;
        } else if confirm.eq_ignore_ascii_case("n\n") || confirm == "\n" || confirm.is_empty() {
            return Ok(!thumbnails.is_empty());
        }
    }
}
