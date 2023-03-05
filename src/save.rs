//! ## `binroots::save`
//!
//! Contains the [`SaveError`][`crate::save::SaveError`] struct and the [`Save`][`crate::save::Save`] trait, as well as
//! an implementation of `save` for [`BinrootsField`][`crate::field::BinrootsField`]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use tracing::{info, instrument};

use crate::field::BinrootsField;
use crate::fileserializer::{FileSerializer, SerializerError};

/// Passed to [`Save::save`] to decide which path to save files to
#[derive(Debug)]
pub enum RootType {
    /// Saves to an in-memory location:
    /// - On Unix, `/tmp/<CARGO_PKG_NAME>/`
    /// - On Windows, `%LOCALAPPDATA%\<CARGO_PKG_NAME>\.memcache\`
    InMemory,
    /// Saves to a persistent location:
    /// - On Unix, `$HOME/.cache/<CARGO_PKG_NAME>/`
    /// - On Windows, `%LOCALAPPDATA%\<CARGO_PKG_NAME>\.cache\`
    Persistent,
}

/// Errors during the save process.
#[derive(Debug)]
pub enum SaveError {
    /// Returned when `save` encounters an error during the recursive creation of a folder structure
    CreateDirectoryError {
        /// The path where `save` attempted to create folders
        path: PathBuf,
        /// The resulting IO error kind.
        ///
        /// See [`std::io::ErrorKind`]
        kind: std::io::ErrorKind,
    },
    /// Returned when `save` fails to call [`std::fs::File::create`]
    CreateFileError {
        /// The path where `save` attempted to create a file
        path: PathBuf,
        /// The resulting IO error kind.
        ///
        /// See [`std::io::ErrorKind`]
        kind: std::io::ErrorKind,
    },
    /// Returned when save fails to write to a file that's already been `create`d
    /// - `contents` will only appear when reporting the error with `Debug`
    WriteFileError {
        /// The path where `save` attempted to write to a file
        path: PathBuf,
        /// The contents that `save` attempted to write into the file. Won't be reported when [`Display`][`std::fmt::Display`]ing the error.
        contents: Vec<u8>,
        /// The resulting IO error kind.
        ///
        /// See [`std::io::ErrorKind`]
        kind: std::io::ErrorKind,
    },
    /// An error caught during binroots's serialization process.
    ///
    /// See [`SerializerError`][`crate::fileserializer::SerializerError`]
    SerializeError(SerializerError),
    /// An error caught while initializing retrieving the project's root directory.
    ///
    /// See [`RootLocationError`]
    RootLocationError(RootLocationError),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::CreateDirectoryError { path, kind } =>
                    format!("Failed to create directory at {path:?} during save; {kind}"),
                Self::CreateFileError { path, kind } =>
                    format!("Failed to create (open) file at {path:?} during save; {kind}"),
                Self::WriteFileError { path, kind, .. } =>
                    format!("Faile to write to {path:?} during save; {kind}"),
                Self::SerializeError(e) => format!("Failed to serialize during save: {e}"),
                Self::RootLocationError(e) => format!("{e}"),
            }
        )
    }
}

impl std::error::Error for SaveError {}

/// Provides data with the ability to save to the disk.
///
/// See [`binroots_struct`][`crate::binroots_struct`] and [`binroots_enum`][`crate::binroots_enum`] for constructing more flexible saveable types.
///
/// ## Example
///
/// ```
/// use binroots::Serialize;
/// use binroots::save::{RootType, Save, SaveError};
///
/// #[derive(Serialize)]
/// enum Activity {
///     Nothing,
///     Playing(String),
/// }
///
/// #[derive(Serialize)]
/// struct MyStruct {
///     field1: String,
///     field2: bool,
///     field3: Activity,
/// }
///
/// fn main() -> Result<(), SaveError> {
///     let me = MyStruct {
///         field1: "Hello".into(),
///         field2: true,
///         field3: Activity::Playing("hideo kame".into()),
///     };
///
///     me.save("mystruct", RootType::InMemory)?;
///
///     // Resulting file structure on Unix:
///     // /tmp/<crate name>/mystruct
///     // ├── field1                => "Hello"
///     // ├── field2                => "true"
///     // ├── field3                => "Playing"
///     // └── field3.value          => "hideo kame"
///
///     Ok(())
/// }
pub trait Save {
    /// [`Serialize`][`serde::Serialize`]s and saves data to "[BINROOTS_DIR][`crate::BINROOTS_DIR`]/\<root\>"
    ///
    /// See [`Save`][`crate::save::Save`] for an example of how to use it.
    fn save<P: Into<PathBuf>>(&self, root: P, location: RootType) -> Result<(), SaveError>;
}

impl<T: Serialize> Save for T {
    fn save<P: Into<PathBuf>>(&self, root: P, location: RootType) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        self.serialize(&mut serializer)
            .map_err(|e| SaveError::SerializeError(e))?;

        save_root(serializer, root.into(), location)
    }
}

impl<const N: &'static str, T: Serialize> BinrootsField<N, T> {
    /// [`Serialize`][`serde::Serialize`]s and saves data to "[BINROOTS_DIR][`crate::BINROOTS_DIR`]/\<root\>"
    ///
    /// Modifies the root save path by appending `BinrootsField::N` (generated as the field name by [`binroots::binroots_struct`][`crate::binroots_struct`])
    pub fn save<P: Into<PathBuf>>(&self, root: P, location: RootType) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        serializer.root = format!("/{N}");
        self.value
            .serialize(&mut serializer)
            .map_err(|e| SaveError::SerializeError(e))?;

        save_root(serializer, root.into(), location)
    }
}

#[instrument]
pub(crate) fn save_root(
    serializer: FileSerializer,
    root: PathBuf,
    location: RootType,
) -> Result<(), SaveError> {
    let path = root_location(location)
        .map_err(|e| SaveError::RootLocationError(e))?
        .join(root);

    std::fs::create_dir_all(&path).map_err(|e| SaveError::CreateDirectoryError {
        path: path.clone(),
        kind: e.kind(),
    })?;

    for file in serializer.output {
        let ext = if let Some(ext) = &file.variant {
            format!(".{ext}")
        } else {
            format!("")
        };

        let file_path = path.join(if let Some(name) = &file.name {
            if file.path.ends_with(name.as_str()) {
                format!("{}{}", &file.path.trim_start_matches("/"), ext)
            } else {
                format!("{}/{}{}", &file.path.trim_start_matches("/"), name, ext)
            }
        } else {
            format!("{}{}", &file.path.trim_start_matches("/"), ext)
        });

        info!(
            "Saving {:?} as a {}",
            file_path,
            if file.is_path { "path" } else { "file" }
        );

        if file_path == path || file.is_path {
            std::fs::create_dir_all(file_path.clone()).map_err(|e| {
                SaveError::CreateDirectoryError {
                    path: file_path,
                    kind: e.kind(),
                }
            })?;
            continue;
        }

        let mut file_tgt = File::create(&file_path).map_err(|e| SaveError::CreateFileError {
            path: file_path.clone(),
            kind: e.kind(),
        })?;

        file_tgt
            .write(&file.output)
            .map_err(|e| SaveError::WriteFileError {
                path: file_path,
                contents: file.output,
                kind: e.kind(),
            })?;
    }

    Ok(())
}

/// Errors during [`root_location`]
#[derive(Debug)]
pub enum RootLocationError {
    /// Returned when [`root_location`] fails to resolve a path
    PathBufError(std::convert::Infallible),
    /// Returned when [`root_location`] fails to retrieve a runtime environment variable
    GetVarError(std::env::VarError),
    /// Returned when [`root_location`] encounters an error during the recursive creation of a folder structure
    CreateDirectoryError {
        /// The path where [`root_location`] attempted to create folders
        path: PathBuf,
        /// The resulting IO error kind.
        ///
        /// See [`std::io::ErrorKind`]
        kind: std::io::ErrorKind,
    },
}

impl std::fmt::Display for RootLocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PathBufError(inf) =>
                    format!("Failed while retrieving the program's root directory: {inf}"),
                Self::GetVarError(ve) =>
                    format!("Failed to get an environment variable while retrieving the program's root directory: {ve}"),
                Self::CreateDirectoryError { path, kind } =>
                    format!("Failed to create (open) file at {path:?} while attempting to initialize the program's root directory; {kind}"),
            }
        )
    }
}

/// Initializes and returns the active program root directory, the folder where files are stored when calling [`Save::save`][`crate::save::Save::save`]
/// - On Windows, the path will always be `%LOCALAPPDATA%\<CARGO_PKG_NAME>\cache` regardless of `location`, since in-memory folders on Windows are inpossible with safe rust.
/// - On Unix with [`RootType::InMemory`], `/tmp/<CARGO_PKG_NAME>/`
/// - On Unix with [`RootType::Persistent`], `$HOME/.cache/<CARGO_PKG_NAME>/`
///
/// CARGO_PKG_NAME is generated during compile-time using the [`env`] macro.
pub fn root_location(location: RootType) -> Result<PathBuf, RootLocationError> {
    use std::str::FromStr;

    #[cfg(target_family = "unix")]
    let path = match location {
        RootType::InMemory => PathBuf::from_str(&format!("/tmp/{}", env!("CARGO_PKG_NAME")))
            .map_err(|e| RootLocationError::PathBufError(e)),
        RootType::Persistent => PathBuf::from_str(&format!(
            "{}/.cache/{}",
            std::env::var("HOME").map_err(|e| RootLocationError::GetVarError(e))?,
            env!("CARGO_PKG_NAME")
        ))
        .map_err(|e| RootLocationError::PathBufError(e)),
    }?;

    #[cfg(target_family = "windows")]
    let path = PathBuf::from_str(&format!(
        "{}\\{}\\.cache",
        std::env::var("LOCALAPPDATA").map_err(|e| RootLocationError::GetVarError(e))?,
        env!("CARGO_PKG_NAME")
    ))
    .map_err(|e| RootLocationError::PathBufError(e))?;

    std::fs::create_dir_all(path.clone()).map_err(|e| RootLocationError::CreateDirectoryError {
        path: path.clone(),
        kind: e.kind(),
    })?;

    Ok(path)
}
