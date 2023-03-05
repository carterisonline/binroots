#![feature(adt_const_params)]

pub mod field;
pub mod fileserializer;
pub mod save;

pub use binroots_proc_macros::*;
pub use serde::Serialize;

use once_cell::sync::Lazy;
use std::fs;

#[cfg(target_family = "unix")]
pub static BINROOTS_DIR: Lazy<std::path::PathBuf> =
    Lazy::new(|| init_binroots_dir(format!("/tmp/{}", env!("CARGO_PKG_NAME"))).into());

#[cfg(target_family = "windows")]
pub static BINROOTS_DIR: Lazy<std::path::PathBuf> = Lazy::new(|| {
    init_binroots_dir(format!(
        "{}\\{}\\cache",
        std::env::var("LOCALAPPDATA").unwrap(),
        env!("CARGO_PKG_NAME")
    ))
    .into()
});

fn init_binroots_dir(dir: String) -> String {
    fs::create_dir_all(&dir).unwrap();
    dir
}
