use log::*;

use thumbs_rs::Thumbnail;

pub mod cleanup;
pub mod cli;
pub mod delete;
pub mod interactive;
pub mod locate;


pub(crate) fn cached_delete(thumbnails: &[Thumbnail]) {
    for p in thumbnails {
        match p.delete() {
            Ok(_) => (),
            Err(e) => warn!("{e}"),
        }
    }

    show!("Deleted {} thumbnail(s).", thumbnails.len());
    ()
}


#[macro_export]
macro_rules! show {
    ($level:ident, $($a:tt)*) => {
        if log::log_enabled!(log::Level::$level) {
            println!($($a)*);
        }
    };
    ($($a:tt)*) => {
        if log::log_enabled!(log::Level::Error) {
            println!($($a)*);
        }
    }
}
