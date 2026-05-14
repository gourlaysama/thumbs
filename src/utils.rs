use std::io::{Error, Write};

use flexi_logger::{AdaptiveFormat, DeferredNow, style};
use log::Record;

pub const ADAPTIVE_LOG_FORMAT: AdaptiveFormat = AdaptiveFormat::Custom(default_format, colored_default_format);

pub fn syslog_prefixed_format(
    w: &mut dyn Write,
    _now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), Error> {
    let level = record.level();
    let prefix = match level {
        log::Level::Error => "<3>",
        log::Level::Warn => "<5>",
        log::Level::Info => "<6>",
        log::Level::Debug | log::Level::Trace => "<7>",
    };

    if let Some(m) = record.module_path() {
        if m.starts_with("thumbs") {
            return write!(w, "{prefix}{}", record.args());
        }
    }

    write!(
        w,
        "{prefix}[{}] ",
        record.module_path().unwrap_or("<unnamed>"),
    )?;

    write!(w, "{}", record.args())
}

pub fn default_format(
    w: &mut dyn Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), Error> {
    let level = record.level();
    if let Some(m) = record.module_path() {
        if m.starts_with("thumbs") {
            return write!(w, "{} {}", level, record.args());
        }
    }

    write!(
        w,
        "{} [{}] ",
        level,
        record.module_path().unwrap_or("<unnamed>"),
    )?;

    write!(w, "{}", record.args())
}

pub fn colored_default_format(
    w: &mut dyn Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), Error> {
    let level = record.level();

    if let Some(m) = record.module_path() {
        if m.starts_with("thumbs") {
            return write!(
                w,
                "{} {}",
                style(level).paint(level.to_string()),
                style(level).paint(record.args().to_string())
            );
        }
    }

    write!(
        w,
        "{} [{}] ",
        style(level).paint(level.to_string()),
        record.module_path().unwrap_or("<unnamed>"),
    )?;

    write!(w, "{}", style(level).paint(record.args().to_string()))
}
