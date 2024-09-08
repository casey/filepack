use super::*;

#[derive(Debug)]
pub(crate) struct DisplayPath(PathBuf);

impl Display for DisplayPath {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.display().fmt(f)
  }
}

impl<T: AsRef<str>> From<T> for DisplayPath {
  fn from(path: T) -> Self {
    Self(Path::new(path.as_ref()).lexiclean())
  }
}
