use {super::*, pretty_assertions::assert_eq, regex::Regex};

enum Expected {
  Regex(Regex),
  String(String),
}

pub(crate) struct Test {
  args: Vec<String>,
  current_dir: Option<String>,
  data_dir: Option<String>,
  env: Vec<(String, String)>,
  stderr: Option<Expected>,
  stdin: Option<String>,
  stdout: Option<Expected>,
  tempdir: tempfile::TempDir,
}

impl Test {
  pub(crate) fn args(mut self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
    for arg in args {
      self.args.push(arg.as_ref().into());
    }
    self
  }

  pub(crate) fn assert_file(self, path: &str, expected: impl AsRef<str>) -> Self {
    let actual = fs::read_to_string(self.tempdir.path().join(path)).unwrap();
    assert_eq!(actual.trim(), expected.as_ref().trim());
    self
  }

  pub(crate) fn assert_file_regex(self, path: &str, pattern: &str) -> Self {
    let actual = fs::read_to_string(self.tempdir.path().join(path)).unwrap();
    let regex = Regex::new(&format!("^(?s){pattern}$")).unwrap();
    assert!(
      regex.is_match(&actual),
      "file content did not match regex\n   actual: {actual}\n    regex: {}",
      regex.as_str()
    );
    self
  }

  pub(crate) fn create_dir(self, path: &str) -> Self {
    fs::create_dir_all(self.tempdir.path().join(path)).unwrap();
    self
  }

  pub(crate) fn current_dir(mut self, path: &str) -> Self {
    self.current_dir = Some(path.into());
    self
  }

  pub(crate) fn data_dir(mut self, path: &str) -> Self {
    self.data_dir = Some(path.into());
    self
  }

  pub(crate) fn env(mut self, key: &str, value: &str) -> Self {
    self.env.push((key.into(), value.into()));
    self
  }

  pub(crate) fn failure(self) -> Self {
    self.run(1)
  }

  pub(crate) fn path(&self) -> camino::Utf8PathBuf {
    camino::Utf8PathBuf::from_path_buf(self.tempdir.path().into()).unwrap()
  }

  pub(crate) fn read(&self, path: &str) -> String {
    fs::read_to_string(self.tempdir.path().join(path))
      .unwrap()
      .trim()
      .into()
  }

  pub(crate) fn rename(self, from: &str, to: &str) -> Self {
    fs::rename(self.tempdir.path().join(from), self.tempdir.path().join(to)).unwrap();
    self
  }

  pub(crate) fn remove_dir(self, path: &str) -> Self {
    fs::remove_dir(self.tempdir.path().join(path)).unwrap();
    self
  }

  pub(crate) fn remove_file(self, path: &str) -> Self {
    fs::remove_file(self.tempdir.path().join(path)).unwrap();
    self
  }

  fn run(self, code: i32) -> Self {
    let mut command = cargo_bin_cmd!("filepack");

    let current_dir = if let Some(ref subdir) = self.current_dir {
      self.tempdir.path().join(subdir)
    } else {
      self.tempdir.path().into()
    };

    command.current_dir(current_dir);

    let data_dir = if let Some(ref subdir) = self.data_dir {
      self.tempdir.path().join(subdir)
    } else {
      self.tempdir.path().into()
    };

    command.env("FILEPACK_DATA_DIR", data_dir);

    for (key, value) in &self.env {
      command.env(key, value);
    }

    command.args(&self.args);

    if let Some(ref stdin_content) = self.stdin {
      command.write_stdin(stdin_content.as_str());
    }

    let output = command.output().unwrap();

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

  pub(crate) fn stdin(mut self, stdin: &str) -> Self {
    self.stdin = Some(stdin.into());
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

  pub(crate) fn stdout_regex(mut self, pattern: &str) -> Self {
    self.stdout = Some(Expected::Regex(
      Regex::new(&format!("^(?s){pattern}$")).unwrap(),
    ));
    self
  }

  pub(crate) fn success(self) -> Self {
    self.run(0)
  }

  pub(crate) fn symlink(self, target: &str, link: &str) -> Self {
    let link_path = self.tempdir.path().join(link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link_path).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(target, link_path).unwrap();
    self
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
      current_dir: None,
      data_dir: None,
      env: Vec::new(),
      stderr: None,
      stdin: None,
      stdout: None,
      tempdir,
    }
  }
}
