use super::*;

pub(crate) trait Utf8PathExt {
  fn is_empty(&self) -> bool;

  fn join_case_sensitive(&self, name: &'static str) -> Result<Utf8PathBuf>;

  fn lexiclean(&self) -> Utf8PathBuf;
}

impl Utf8PathExt for Utf8Path {
  fn is_empty(&self) -> bool {
    self.as_str().is_empty()
  }

  fn join_case_sensitive(&self, name: &'static str) -> Result<Utf8PathBuf> {
    check_case(self, &[name])?;
    Ok(self.join(name))
  }

  fn lexiclean(&self) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(self.as_std_path().lexiclean()).unwrap()
  }
}
