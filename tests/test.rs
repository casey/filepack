use {super::*, pretty_assertions::assert_eq, regex::Regex};

enum Expected {
  Regex(Regex),
  String(String),
}

pub(crate) struct Test {
  args: Vec<String>,
  env: Vec<(String, String)>,
  stderr: Option<Expected>,
  stdout: Option<Expected>,
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
      env: Vec::new(),
      stderr: None,
      stdout: None,
      tempdir,
    }
  }

  pub(crate) fn arg(mut self, arg: impl AsRef<str>) -> Self {
    self.args.push(arg.as_ref().into());
    self
  }

  pub(crate) fn args(mut self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
    for arg in args {
      self.args.push(arg.as_ref().into());
    }
    self
  }

  pub(crate) fn create_dir(self, path: &str) -> Self {
    fs::create_dir_all(self.tempdir.path().join(path)).unwrap();
    self
  }

  pub(crate) fn env(mut self, key: &str, value: impl AsRef<Path>) -> Self {
    self.env.push((key.into(), value.as_ref().display().to_string()));
    self
  }

  pub(crate) fn failure(self) -> Self {
    self.run(1)
  }

  pub(crate) fn read(&self, path: &str) -> String {
    fs::read_to_string(self.tempdir.path().join(path))
      .unwrap()
      .trim()
      .into()
  }

  pub(crate) fn remove_dir(self, path: &str) -> Self {
    fs::remove_dir(self.tempdir.path().join(path)).unwrap();
    self
  }

  fn run(self, code: i32) -> Self {
    let mut command = cargo_bin_cmd!("filepack");

    command.current_dir(self.tempdir.path());
    command.env("FILEPACK_DATA_DIR", self.tempdir.path());

    for (key, value) in &self.env {
      command.env(key, value);
    }

    let output = command.args(&self.args).output().unwrap();

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

    let stdout = str::from_utf8(&output.stdout).unwrap();

    match &self.stdout {
      Some(Expected::String(expected)) => assert_eq!(stdout, expected),
      Some(Expected::Regex(regex)) => assert!(
        regex.is_match(stdout),
        "stdout did not match regex\n   stdout: {stdout}\n    regex: {}",
        regex.as_str()
      ),
      None => assert!(output.stdout.is_empty()),
    }

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

  pub(crate) fn stdout(mut self, stdout: impl AsRef<[u8]>) -> Self {
    self.stdout = Some(Expected::String(
      String::from_utf8(stdout.as_ref().to_vec()).unwrap(),
    ));
    self
  }

  pub(crate) fn success(self) -> Self {
    self.run(0)
  }

  pub(crate) fn tempdir_path(&self) -> &Path {
    self.tempdir.path()
  }

  pub(crate) fn touch(self, path: impl AsRef<Path>) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, []).unwrap();
    self
  }

  pub(crate) fn write(self, path: &str, content: impl AsRef<[u8]>) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content.as_ref()).unwrap();
    self
  }
}
