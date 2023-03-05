use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use tracing::{info, instrument};

use crate::field::BinrootsField;
use crate::fileserializer::{FileSerializer, SerializerError};
use crate::BINROOTS_DIR;

#[derive(Debug)]
pub enum SaveError {
    CreateDirectoryError {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    CreateFileError {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    WriteFileError {
        path: PathBuf,
        contents: Vec<u8>,
        kind: std::io::ErrorKind,
    },
    SerializeError(SerializerError),
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
            }
        )
    }
}

impl std::error::Error for SaveError {}

pub trait Save {
    fn save<P: Into<PathBuf>>(&self, root: P) -> Result<(), SaveError>;
}

impl<T: Serialize> Save for T {
    fn save<P: Into<PathBuf>>(&self, root: P) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        self.serialize(&mut serializer)
            .map_err(|e| SaveError::SerializeError(e))?;

        save_root(serializer, root.into())
    }
}

impl<const N: &'static str, T: Serialize> BinrootsField<N, T> {
    pub fn save<P: Into<PathBuf>>(&self, root: P) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        serializer.root = format!("/{N}");
        self.value
            .serialize(&mut serializer)
            .map_err(|e| SaveError::SerializeError(e))?;

        save_root(serializer, root.into())
    }
}

#[instrument]
pub(crate) fn save_root(serializer: FileSerializer, root: PathBuf) -> Result<(), SaveError> {
    let path = (*BINROOTS_DIR).join(root);
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
