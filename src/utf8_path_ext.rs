use super::*;

pub(crate) trait Utf8PathExt {
  fn containing_dir(&self) -> &Utf8Path;

  fn is_empty(&self) -> bool;

  fn join_case_sensitive(&self, name: &'static str) -> Result<Utf8PathBuf>;

  fn lexiclean(&self) -> Utf8PathBuf;
}

impl Utf8PathExt for Utf8Path {
  fn containing_dir(&self) -> &Utf8Path {
    let parent = self.parent().unwrap();

    if parent.is_empty() {
      Utf8Path::new(".")
    } else {
      parent
    }
  }

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn containing_dir() {
    assert_eq!(Utf8Path::new("foo").containing_dir(), ".");
    assert_eq!(Utf8Path::new("foo/bar").containing_dir(), "foo");
  }

  #[test]
  #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
  fn containing_dir_without_parent() {
    Utf8Path::new("/").containing_dir();
  }
}
