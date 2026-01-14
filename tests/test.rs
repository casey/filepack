use {super::*, pretty_assertions::assert_eq};

pub(crate) struct Test {
  args: Vec<&'static str>,
  stderr: Option<String>,
  tempdir: tempfile::TempDir,
}

impl Test {
  pub(crate) fn new() -> Self {
    Self::foo(
      tempfile::Builder::new()
        .prefix("filepack-test-tempdir")
        .tempdir()
        .unwrap(),
    )
  }

  fn foo(tempdir: tempfile::TempDir) -> Self {
    Self {
      args: Vec::new(),
      stderr: None,
      tempdir,
    }
  }

  pub(crate) fn args(mut self, args: impl AsRef<[&'static str]>) -> Self {
    for &arg in args.as_ref() {
      self.args.push(arg.into());
    }

    self
  }

  pub(crate) fn failure(self) -> Self {
    self.run(1)
  }

  pub(crate) fn success(self) -> Self {
    self.run(0)
  }

  fn run(self, code: i32) -> Self {
    let mut command = cargo_bin_cmd!("filepack");

    command.current_dir(self.tempdir.path());

    let output = command.args(self.args).output().unwrap();

    assert_eq!(output.status.code(), Some(code));

    if let Some(expected) = self.stderr {
      let actual = str::from_utf8(&output.stderr).unwrap();
      assert_eq!(actual, expected);
    } else {
      assert!(output.stderr.is_empty());
    }

    assert!(output.stdout.is_empty());

    Self::foo(self.tempdir)
  }

  pub(crate) fn stderr(mut self, stderr: &str) -> Self {
    self.stderr = Some(stderr.into());
    self
  }

  pub(crate) fn write(self, path: &str, content: String) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
    self
  }

  pub(crate) fn create_dir(self, path: &str) -> Self {
    fs::create_dir_all(self.tempdir.path().join(path)).unwrap();
    self
  }

  pub(crate) fn touch(self, path: &str) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, []).unwrap();
    self
  }
}
