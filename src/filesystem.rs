use super::*;

#[cfg(all(test, unix))]
pub(crate) fn chmod(path: &Utf8Path, mode: u32) -> Result {
  use std::os::unix::fs::PermissionsExt;
  fs::set_permissions(path, Permissions::from_mode(mode)).context(error::FilesystemIo { path })
}

#[cfg(all(test, not(unix)))]
pub(crate) fn chmod(_path: &Utf8Path, _mode: u32) -> Result {
  Ok(())
}

pub(crate) fn create_dir_all(path: &Utf8Path) -> Result {
  fs::create_dir_all(path).context(error::FilesystemIo { path })
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
pub(crate) fn create_dir_all_with_mode(path: &Utf8Path, _mode: u32) -> Result {
  create_dir_all(path)
}

pub(crate) fn exists(path: &Utf8Path) -> Result<bool> {
  path.try_exists().context(error::FilesystemIo { path })
}

pub(crate) fn metadata(path: &Utf8Path) -> Result<fs::Metadata> {
  fs::metadata(path).context(error::FilesystemIo { path })
}

pub(crate) fn mode(path: &Utf8Path) -> Result<Mode> {
  Ok(metadata(path)?.permissions().into())
}

pub(crate) fn open(path: &Utf8Path) -> Result<fs::File> {
  fs::File::open(path).context(error::FilesystemIo { path })
}

pub(crate) fn read(path: &Utf8Path) -> Result<Vec<u8>> {
  fs::read(path).context(error::FilesystemIo { path })
}

pub(crate) fn read_opt(path: &Utf8Path) -> Result<Option<Vec<u8>>> {
  match fs::read(path) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
    result => result.map(Some).context(error::FilesystemIo { path }),
  }
}

pub(crate) fn read_to_string(path: impl AsRef<Utf8Path>) -> Result<String> {
  fs::read_to_string(path.as_ref()).context(error::FilesystemIo {
    path: path.as_ref(),
  })
}

pub(crate) fn read_to_string_opt(path: &Utf8Path) -> Result<Option<String>> {
  match fs::read_to_string(path) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
    result => result.map(Some).context(error::FilesystemIo { path }),
  }
}

pub(crate) fn write(path: &Utf8Path, contents: impl AsRef<[u8]>) -> Result {
  fs::write(path, contents).context(error::FilesystemIo { path })
}

pub(crate) fn write_new(path: &Utf8Path, contents: impl AsRef<[u8]>) -> Result {
  OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(path)
    .and_then(|mut file| file.write_all(contents.as_ref()))
    .context(error::FilesystemIo { path })
}

#[cfg(unix)]
pub(crate) fn write_with_mode(path: &Utf8Path, contents: impl AsRef<[u8]>, mode: u32) -> Result {
  use std::os::unix::fs::OpenOptionsExt;

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
