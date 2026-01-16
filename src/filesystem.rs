use super::*;

#[cfg(all(test, unix))]
pub(crate) fn chmod(path: &Utf8Path, mode: u32) -> Result {
  use std::os::unix::fs::PermissionsExt;
  std::fs::set_permissions(path, Permissions::from_mode(mode)).context(error::FilesystemIo { path })
}

#[cfg(all(test, not(unix)))]
pub(crate) fn chmod(_path: &Utf8Path, _mode: u32) -> Result {
  Ok(())
}

pub(crate) fn create_dir_all(path: &Utf8Path) -> Result {
  std::fs::create_dir_all(path).context(error::FilesystemIo { path })
}

#[cfg(unix)]
pub(crate) fn create_dir_all_with_mode(path: &Utf8Path, mode: u32) -> Result {
  use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt};

  if let Some(parent) = path.parent() {
    create_dir_all(parent)?;
  }

  DirBuilder::new()
    .mode(mode)
    .create(path)
    .context(error::FilesystemIo { path })
}

#[cfg(not(unix))]
pub(crate) fn create_dir_all_with_mode(path: &Utf8Path, _mode: u32) -> Resul {
  create_dir_all(path)
}

pub(crate) fn exists(path: &Utf8Path) -> Result<bool> {
  path.try_exists().context(error::FilesystemIo { path })
}

pub(crate) fn metadata(path: &Utf8Path) -> Result<std::fs::Metadata> {
  std::fs::metadata(path).context(error::FilesystemIo { path })
}

pub(crate) fn mode(path: &Utf8Path) -> Result<Mode> {
  Ok(metadata(path)?.permissions().into())
}

pub(crate) fn read_to_string(path: impl AsRef<Utf8Path>) -> Result<String> {
  std::fs::read_to_string(path.as_ref()).context(error::FilesystemIo {
    path: path.as_ref(),
  })
}

pub(crate) fn read_to_string_opt(path: &Utf8Path) -> Result<Option<String>> {
  match std::fs::read_to_string(path) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
    result => result.map(Some).context(error::FilesystemIo { path }),
  }
}

pub(crate) fn write(path: &Utf8Path, contents: impl AsRef<[u8]>) -> Result {
  std::fs::write(path, contents).context(error::FilesystemIo { path })
}

#[cfg(unix)]
pub(crate) fn write_with_mode(path: &Utf8Path, contents: impl AsRef<[u8]>, mode: u32) -> Result {
  use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt};

  OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .mode(mode)
    .open(path)
    .and_then(|mut file| file.write_all(contents.as_ref()))
    .context(error::FilesystemIo { path })
}

#[cfg(not(unix))]
pub(crate) fn write_with_mode(path: &Utf8Path, contents: impl AsRef<[u8]>, _mode: u32) -> Result {
  write(path, contents)
}
