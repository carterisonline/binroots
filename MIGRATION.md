# Migration

## 0.1.0
### Initial Commit

## 0.1.1
### More appropriate errors during Save::save()

- MOVED `SaveError::CouldNotSerialize` -> `SaveError::SerializeError`
	- Now only returns when there's an error with the serializer
- REMOVED `SaveError::CouldNotSave`
	- Use either `SaveError::CreateFileError` or `SaveError::WriteFileError`
- ADD `SaveError::CreateDirectoryError`
	- Returned when save encounters an error during the recursive creation of a folder structure
- ADD `SaveError::CreateFileError`
	- Returned when save fails to call `File::create`
- ADD `SaveError::WriteFileError`
	- Returned when save fails to write to a file that's already been `create`d
	- `contents` will only appear when reporting the error with `Debug`

## 0.1.2
### Added docs for every public item
(no migration required)

## 0.2.0
### Enabled saving to different paths on Unix
- MOVED `BINROOTS_DIR` -> `save::root_location`
	- Requires a `RootType`
	- Returns a `RootLocationError`
- UPDATED `Save::save` and `BinrootsField::save`
	- Requires a `RootType`
- UPDATED `SaveError`
	- New variant `RootLocationError` for handling errors during root location initialization
- ADD optional `persistent` attribute to `binroots_struct`
	- Use when saving to persistent storage instead of in-memory storage (Unix only)
- ADD `RootLocationError`
	- Contains variants for handling errors during root location initialization