use super::*;

pub(crate) fn create_dir_all(path: &Utf8Path) -> Result<()> {
  std::fs::create_dir_all(path).context(error::FilesystemIo { path })
}

pub(crate) fn exists(path: &Utf8Path) -> Result<bool> {
  path.try_exists().context(error::FilesystemIo { path })
}

pub(crate) fn metadata(path: &Utf8Path) -> Result<std::fs::Metadata> {
  std::fs::metadata(path).context(error::FilesystemIo { path })
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
