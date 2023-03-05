//! # `binroots`
//! Binroots is a (cross-platform!) crate that provides a simple and efficient way to save Rust data structures to disk. It allows you to save each field of a struct or enum variant as a separate file, making it easy to store reactive data, allowing end-users and hackers to watch individual files for changes and automate command-line tools for your app.
//!
//! ## Setting up a struct - see [`binroots_struct`][`crate::binroots_struct`]
//! ```rust
//! use binroots::binroots_struct;
//!
//! # #[binroots::binroots_enum]
//! # enum Activity {
//! #     None, // <- Automatically chosen as the default value
//! #     Playing(String),
//! # }
//!
//! #[binroots_struct]
//! struct Status {
//!     connections: usize,
//!     is_online: bool,
//!     activity: Activity,
//! }
//! ```
//!
//! ## Setting up an enum - see [`binroots_enum`][`crate::binroots_enum`]
//! ```rust
//! use binroots::binroots_enum;
//!
//! #[binroots_enum]
//! enum Activity {
//!     None, // <- Automatically chosen as the default value
//!     Playing(String),
//! }
//! ```
//!
//! ## Saving data - see [`Save::save`][`crate::save::Save::save`] and [`binroots_struct`][`crate::binroots_struct`]
//! ```rust
//! # use binroots::{binroots_enum, binroots_struct};
//! #[binroots_enum]
//! # enum Activity {
//! #     None, // <- Automatically chosen as the default value
//! #     Playing(String),
//! # }
//! # #[binroots_struct]
//! # struct Status {
//! #     connections: usize,
//! #     is_online: bool,
//! #     activity: Activity,
//! # }
//!
//! use binroots::save::SaveError;
//!
//! fn main() -> Result<(), SaveError> {
//!     let mut status = Status::default();
//!
//!     *status.is_online = true;
//!     status.save()?; // <- Saves the entire struct to the disk
//!
//!     *status.activity = Activity::Playing("video gamb".into());
//!     status.activity.save(Status::ROOT_FOLDER)?; // <- Only saves status.activity to the disk
//!
//!     Ok(())
//! }
//! ```

#![feature(adt_const_params)]
#![warn(missing_docs)]

pub mod field;
pub mod fileserializer;
pub mod save;

pub use binroots_proc_macros::*;
pub use serde::Serialize;

use once_cell::sync::Lazy;
use std::fs;

/// The directory to save to when calling [`Save::save`][`crate::save::Save::save`]
/// - On Unix, `/tmp/<CARGO_PKG_NAME>/`
/// - On Windows, `%LOCALAPPDATA%\<CARGO_PKG_NAME>\cache`
///
/// CARGO_PKG_NAME is generated during compile-time using the [`env`] macro.
#[cfg(target_family = "unix")]
pub static BINROOTS_DIR: Lazy<std::path::PathBuf> =
    Lazy::new(|| init_binroots_dir(format!("/tmp/{}", env!("CARGO_PKG_NAME"))).into());

/// The directory to save to when calling [`Save::save`][`crate::save::Save::save`]
/// - On Unix, `/tmp/<CARGO_PKG_NAME>/`
/// - On Windows, `%LOCALAPPDATA%\<CARGO_PKG_NAME>\cache`
///
/// CARGO_PKG_NAME is generated during compile-time using the [`env`] macro.
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
