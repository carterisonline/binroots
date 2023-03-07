//! ## `binroots::save`
//!
//! Contains the [`SaveError`][`crate::save::SaveError`] struct and the [`Save`][`crate::save::Save`] trait, as well as
//! an implementation of `save` for [`BinrootsField`][`crate::field::BinrootsField`]'

use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

use serde::Serialize;

use crate::field::BinrootsField;
use crate::fileserializer::{FileOperationHint, FileSerializer, SerializerError};

/// Passed to [`Save::save`] to decide which path to save files to
#[derive(Debug, Clone)]
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
    /// Returned when `save` fails to call [`std::fs::remove_file`]
    DeleteFileError {
        /// The path where `save` attempted to delete a file
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
                Self::DeleteFileError { path, kind } =>
                    format!("Failed to delete a file at {path:?} during save; {kind}"),
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
    fn save<P: Into<PathBuf>>(&self, root: P, root_type: RootType) -> Result<(), SaveError>;
}

impl<T: Serialize> Save for T {
    fn save<P: Into<PathBuf>>(&self, root: P, root_type: RootType) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        self.serialize(&mut serializer)
            .map_err(|e| SaveError::SerializeError(e))?;

        save_root(serializer, root.into(), root_type)
    }
}

impl<const N: &'static str, T: Serialize> BinrootsField<N, T> {
    /// [`Serialize`][`serde::Serialize`]s and saves data to "[BINROOTS_DIR][`crate::BINROOTS_DIR`]/\<root\>"
    ///
    /// Modifies the root save path by appending `BinrootsField::N` (generated as the field name by [`binroots::binroots_struct`][`crate::binroots_struct`])
    pub fn save<P: Into<PathBuf>>(&self, root: P, root_type: RootType) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        serializer.root = format!("/{N}");
        self.value
            .serialize(&mut serializer)
            .map_err(|e| SaveError::SerializeError(e))?;

        save_root(serializer, root.into(), root_type)
    }
}

pub(crate) fn save_root(
    serializer: FileSerializer,
    root: PathBuf,
    root_type: RootType,
) -> Result<(), SaveError> {
    let path = root_location(root_type.clone())
        .map_err(|e| SaveError::RootLocationError(e))?
        .join(root);

    for file in serializer.output {
        let path = &PathBuf::from(format!(
            "{}{}",
            path.to_string_lossy().trim_end_matches('/'),
            if let Some(folder_variant) = file.folder_variant.clone() {
                format!(".{folder_variant}")
            } else {
                format!("")
            }
        ));

        let file_path = if let Some(name) = &file.name {
            path.join(PathBuf::from(
                format!("{}/{}", &file.path.trim_matches('/'), name).trim_start_matches("/"),
            ))
        } else {
            path.join(PathBuf::from(&file.path.trim_matches('/')))
        };

        if file.hint == FileOperationHint::DeleteValue {
            rmdir(file_path.with_extension("value"))?;
            rm(file_path.with_extension("value"))?;
        }

        if !file.is_path {
            std::fs::create_dir_all(if &file_path != path {
                &path
            } else {
                path.parent().unwrap()
            })
            .map_err(|e| SaveError::CreateDirectoryError {
                path: path.clone(),
                kind: e.kind(),
            })?;

            if file.hint == FileOperationHint::Delete {
                rm(file_path)?;
            } else {
                let filename = format!(
                    "{}{}",
                    file_path.to_string_lossy().trim_end_matches('/'),
                    if let Some(ext) = &file.variant {
                        format!(".{ext}")
                    } else {
                        format!("")
                    }
                );
                save_to(filename.into(), file.output)?;
            }
        } else {
            std::fs::create_dir_all(format!(
                "{}{}",
                file_path.to_string_lossy().trim_end_matches('/'),
                if let Some(ext) = &file.variant {
                    format!(".{ext}")
                } else {
                    format!("")
                }
            ))
            .map_err(|e| SaveError::CreateDirectoryError {
                path: file_path,
                kind: e.kind(),
            })?;
        }
    }

    Ok(())
}

fn rmdir(path: PathBuf) -> Result<(), SaveError> {
    std::fs::remove_dir_all(path.clone()).map_or_else(
        |e| {
            let kind = e.kind();

            match kind {
                ErrorKind::NotFound => Ok(()),
                ErrorKind::NotADirectory => Ok(()),
                _ => Err(SaveError::DeleteFileError { path, kind }),
            }
        },
        |_| Ok(()),
    )
}

fn rm(path: PathBuf) -> Result<(), SaveError> {
    std::fs::remove_file(path.to_string_lossy().trim_end_matches('/')).map_or_else(
        |e| {
            let kind = e.kind();

            match kind {
                ErrorKind::NotFound => Ok(()),
                _ => Err(SaveError::DeleteFileError { path, kind }),
            }
        },
        |_| Ok(()),
    )
}

fn save_to(path: PathBuf, contents: Vec<u8>) -> Result<(), SaveError> {
    let mut file_tgt = File::create(&path).map_err(|e| SaveError::CreateFileError {
        path: path.clone(),
        kind: e.kind(),
    })?;

    file_tgt
        .write(&contents)
        .map_err(|e| SaveError::WriteFileError {
            path,
            contents,
            kind: e.kind(),
        })?;

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
