use super::*;

pub(crate) fn read_to_string(path: &Utf8Path) -> Result<String> {
  std::fs::read_to_string(path).context(error::Io { path })
}

pub(crate) fn metadata(path: &Utf8Path) -> Result<std::fs::Metadata> {
  std::fs::metadata(path).context(error::Io { path })
}

pub(crate) fn write(path: &Utf8Path, contents: &[u8]) -> Result {
  std::fs::write(path, contents).context(error::Io { path })
}

pub(crate) fn exists(path: &Utf8Path) -> Result<bool> {
  path.try_exists().context(error::Io { path })
}
