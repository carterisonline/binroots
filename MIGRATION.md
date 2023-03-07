# Migration

## 0.1.0

### Initial Commit

## 0.1.1

### More appropriate errors during Save::save()

- MOVED `SaveError::CouldNotSerialize` -> `SaveError::SerializeError`
  - Now only returns when there's an error with the serializer
- REMOVED `SaveError::CouldNotSave`
  - Use either `SaveError::CreateFileError` or `SaveError::WriteFileError`
- ADDED `SaveError::CreateDirectoryError`
  - Returned when save encounters an error during the recursive creation of a folder structure
- ADDED `SaveError::CreateFileError`
  - Returned when save fails to call `File::create`
- ADDED `SaveError::WriteFileError`
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
- ADDED `RootLocationError`
  - Contains variants for handling errors during root location initialization

## 0.2.2

## Internal fixes and unit test support

- REMOVED `tracing` as a dependency
- UPDATED `Save::save`
  - HashMaps panic in debug mode when its key is represented by anything other than a whitelisted type. Whitelisted key types include strings, ints, floats, units, and option containing one of those types.
  - HashMaps save `Option::None`, if used as a key, as `<path>/__NONE__`
  - HashMaps save `()`, or unit types, if used as a key, as `<path>/__UNIT__`
  - Enum variants will remove previous "<name>.value" folders/files before writing
  - `None` is now represented as a lack of a file instead of an empty one. Will also remove its previous value if resolved to the same path
  - Properly serializes and saves multi-layer sequence arrays/tuples
- UPDATED `SaveError`
  - New variant `DeleteFileError` for when `save` fails to call `std::fs::remove_file`
- UPDATED `RootType`
  - Now inherits `Clone`
