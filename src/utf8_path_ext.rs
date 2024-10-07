use super::*;

pub(crate) trait Utf8PathExt {
  fn lexiclean(&self) -> Utf8PathBuf;
}

impl Utf8PathExt for Utf8Path {
  fn lexiclean(&self) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(self.as_std_path().lexiclean()).unwrap()
  }
}
