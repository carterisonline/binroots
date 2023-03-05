use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use tracing::{info, instrument};

use crate::field::BinrootsField;
use crate::fileserializer::FileSerializer;
use crate::BINROOTS_DIR;

#[derive(Debug)]
pub enum SaveError {
    CouldNotSave(PathBuf),
    CouldNotSerialize,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::CouldNotSave(path) => format!("Unable to save to {}", path.to_string_lossy()),
                Self::CouldNotSerialize => format!("Unable to serialize"),
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
            .map_err(|_| SaveError::CouldNotSerialize)?;

        save_root(serializer, root.into())
    }
}

impl<const N: &'static str, T: Serialize> BinrootsField<N, T> {
    pub fn save<P: Into<PathBuf>>(&self, root: P) -> Result<(), SaveError> {
        let mut serializer = FileSerializer::default();
        serializer.root = format!("/{N}");
        self.value
            .serialize(&mut serializer)
            .map_err(|_| SaveError::CouldNotSerialize)?;

        save_root(serializer, root.into())
    }
}

#[instrument]
pub(crate) fn save_root(serializer: FileSerializer, root: PathBuf) -> Result<(), SaveError> {
    let path = (*BINROOTS_DIR).join(root);
    std::fs::create_dir_all(&path).map_err(|_| SaveError::CouldNotSerialize)?;

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
            std::fs::create_dir_all(file_path).map_err(|_| SaveError::CouldNotSerialize)?;
            continue;
        }
        let mut file_tgt =
            File::create(&file_path).map_err(|_| SaveError::CouldNotSave(file_path.clone()))?;

        file_tgt
            .write(&file.output)
            .map_err(|_| SaveError::CouldNotSave(file_path))?;
    }

    Ok(())
}
