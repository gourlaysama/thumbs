use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use flexi_logger::LogSpecBuilder;
use log::*;
use std::process::{ExitCode, Termination};
use thumbs::cli::{Command, ProgramOptions};
use thumbs::{run, utils};

#[repr(u8)]
pub enum ThumbsResult {
    OK = 0,
    Error = 1,
    NothingToDo = 125,
}

impl Termination for ThumbsResult {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

fn main() -> ThumbsResult {
    match setup().and_then(run) {
        // Everything ok
        Ok(true) => ThumbsResult::OK,
        // Found nothing to do
        Ok(false) => ThumbsResult::NothingToDo,
        Err(e) => {
            error!("{e}");

            for cause in e.chain().skip(1) {
                error!("cause: {cause}");
            }

            ThumbsResult::Error
        }
    }
}

fn setup() -> Result<Command> {
    let args_matches = ProgramOptions::command().get_matches();
    let args = ProgramOptions::from_arg_matches(&args_matches).unwrap();

    let mut log_spec = LogSpecBuilder::new();
    log_spec.default(args.log_level_with_default(1));
    log_spec.module("thumbs", args.log_level_with_default(2));
    log_spec.module("thumbs_rs", args.log_level_with_default(2));
    log_spec.module("thumbs_rs::thumbnail", args.log_level_with_default(1));

    let mut logger = flexi_logger::Logger::try_with_str("trace")?;

    match std::env::var("THUMBS_SYSLOG_PREFIXED") {
        Ok(v) if v != "0" => {
            logger = logger.format(utils::syslog_prefixed_format);
        }
        _ => match std::env::var("NO_COLOR") {
            Ok(v) if v != "0" => {
                logger = logger.format(utils::default_format);
            }
            _ => {
                logger = logger.adaptive_format_for_stderr(utils::ADAPTIVE_LOG_FORMAT);
            }
        },
    }

    let _h = logger.start()?;
    _h.set_new_spec(log_spec.finalize());

    let cmd = args
        .cmd
        .expect("unexpected command, should have been caught by clap.");

    Ok(cmd)
}
