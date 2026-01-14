use {super::*, pretty_assertions::assert_eq, regex::Regex};

enum Expected {
  Regex(Regex),
  String(String),
}

pub(crate) struct Test {
  args: Vec<&'static str>,
  stderr: Option<Expected>,
  tempdir: tempfile::TempDir,
}

impl Test {
  pub(crate) fn new() -> Self {
    Self::with_tempdir(
      tempfile::Builder::new()
        .prefix("filepack-test-tempdir")
        .tempdir()
        .unwrap(),
    )
  }

  fn with_tempdir(tempdir: tempfile::TempDir) -> Self {
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

  pub(crate) fn create_dir(self, path: &str) -> Self {
    fs::create_dir_all(self.tempdir.path().join(path)).unwrap();
    self
  }

  pub(crate) fn failure(self) -> Self {
    self.run(1)
  }

  fn run(self, code: i32) -> Self {
    let mut command = cargo_bin_cmd!("filepack");

    command.current_dir(self.tempdir.path());

    let output = command.args(self.args).output().unwrap();

    assert_eq!(output.status.code(), Some(code));

    let stderr = str::from_utf8(&output.stderr).unwrap();

    match &self.stderr {
      Some(Expected::String(expected)) => assert_eq!(stderr, expected),
      Some(Expected::Regex(regex)) => assert!(
        regex.is_match(stderr),
        "stderr did not match regex\n   stderr: {stderr}\n    regex: {}",
        regex.as_str()
      ),
      None => assert!(output.stderr.is_empty()),
    }

    assert!(output.stdout.is_empty());

    Self::with_tempdir(self.tempdir)
  }

  pub(crate) fn stderr(mut self, stderr: &str) -> Self {
    self.stderr = Some(Expected::String(stderr.into()));
    self
  }

  pub(crate) fn stderr_regex(mut self, pattern: &str) -> Self {
    self.stderr = Some(Expected::Regex(
      Regex::new(&format!("^(?s){pattern}$")).unwrap(),
    ));
    self
  }

  pub(crate) fn success(self) -> Self {
    self.run(0)
  }

  pub(crate) fn touch(self, path: &str) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, []).unwrap();
    self
  }

  pub(crate) fn write(self, path: &str, content: impl AsRef<str>) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content.as_ref()).unwrap();
    self
  }
}
