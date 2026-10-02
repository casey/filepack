use super::*;

#[cfg(all(test, unix))]
pub(crate) fn chmod(path: &Utf8Path, mode: u32) -> FilesystemResult {
  use std::os::unix::fs::PermissionsExt;
  fs::set_permissions(path, Permissions::from_mode(mode)).context(filesystem_error::Io { path })
}

#[cfg(all(test, not(unix)))]
pub(crate) fn chmod(_path: &Utf8Path, _mode: u32) -> FilesystemResult {
  Ok(())
}

pub(crate) fn create_dir_all(path: &Utf8Path) -> FilesystemResult {
  fs::create_dir_all(path).context(filesystem_error::Io { path })
}

#[cfg(unix)]
pub(crate) fn create_dir_all_with_mode(path: &Utf8Path, mode: u32) -> FilesystemResult {
  use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt};

  if let Some(parent) = path.parent() {
    create_dir_all(parent)?;
  }

  DirBuilder::new()
    .mode(mode)
    .create(path)
    .context(filesystem_error::Io { path })
}

#[cfg(not(unix))]
pub(crate) fn create_dir_all_with_mode(path: &Utf8Path, _mode: u32) -> FilesystemResult {
  create_dir_all(path)
}

pub(crate) fn exists(path: &Utf8Path) -> FilesystemResult<bool> {
  path.try_exists().context(filesystem_error::Io { path })
}

pub(crate) fn metadata(path: &Utf8Path) -> FilesystemResult<fs::Metadata> {
  fs::metadata(path).context(filesystem_error::Io { path })
}

pub(crate) fn mode(path: &Utf8Path) -> FilesystemResult<Mode> {
  Ok(metadata(path)?.permissions().into())
}

pub(crate) fn open(path: &Utf8Path) -> FilesystemResult<fs::File> {
  fs::File::open(path).context(filesystem_error::Io { path })
}

pub(crate) fn read(path: &Utf8Path) -> FilesystemResult<Vec<u8>> {
  fs::read(path).context(filesystem_error::Io { path })
}

pub(crate) fn read_opt(path: &Utf8Path) -> FilesystemResult<Option<Vec<u8>>> {
  match fs::read(path) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
    result => result.map(Some).context(filesystem_error::Io { path }),
  }
}

pub(crate) fn read_to_string(path: impl AsRef<Utf8Path>) -> FilesystemResult<String> {
  fs::read_to_string(path.as_ref()).context(filesystem_error::Io {
    path: path.as_ref(),
  })
}

pub(crate) fn read_to_string_opt(path: &Utf8Path) -> FilesystemResult<Option<String>> {
  match fs::read_to_string(path) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
    result => result.map(Some).context(filesystem_error::Io { path }),
  }
}

pub(crate) fn write(path: &Utf8Path, contents: impl AsRef<[u8]>) -> FilesystemResult {
  fs::write(path, contents).context(filesystem_error::Io { path })
}

#[cfg(unix)]
pub(crate) fn write_with_mode(
  path: &Utf8Path,
  contents: impl AsRef<[u8]>,
  mode: u32,
) -> FilesystemResult {
  use {fs::OpenOptions, std::os::unix::fs::OpenOptionsExt};

  OpenOptions::new()
    .write(true)
    .create_new(true)
    .mode(mode)
    .open(path)
    .and_then(|mut file| file.write_all(contents.as_ref()))
    .context(filesystem_error::Io { path })
}

#[cfg(not(unix))]
pub(crate) fn write_with_mode(
  path: &Utf8Path,
  contents: impl AsRef<[u8]>,
  _mode: u32,
) -> FilesystemResult {
  write(path, contents)
}
