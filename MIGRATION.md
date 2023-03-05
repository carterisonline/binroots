# Migration

## 0.1.0
Initial Commit

## 0.1.1
### More appropriate errors during Save::save()

- CHANGED `SaveError::CouldNotSerialize` -> `SaveError::SerializeError(SerializerError)`
	- Now only returns when there's an error with the serializer
- REMOVED `SaveError::CouldNotSave(PathBuf)`
	- Use either `SaveError::CreateFileError` or `SaveError::WriteFileError`
- ADD `SaveError::CreateDirectoryError { path: PathBuf, kind: std::io::ErrorKind }`
	- Returned when save encounters an error during the recursive creation of a folder structure
- ADD `SaveError::CreateFileError { path: PathBuf, kind: std::io::ErrorKind}`
	- Returned when save fails to call `std::fs::File::create`
- ADD `SaveError::WriteFileError { path: PathBuf, contents: Vec<u8>, kind: std::io::ErrorKind }`
	- Returned when save fails to write to a file that's already been `create`d
	- `contents` will only appear when reporting the error with `Debug`

## 0.1.2
### Added docs for every public item
(no migration required)