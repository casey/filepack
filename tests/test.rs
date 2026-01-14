use {super::*, pretty_assertions::assert_eq};

pub(crate) struct Test {
  args: Vec<String>,
  current_dir: Option<String>,
  data_dir: Option<String>,
  env: BTreeMap<String, String>,
  files: BTreeMap<String, Expected>,
  stderr: Expected,
  stdin: Option<String>,
  stdout: Expected,
  tempdir: tempfile::TempDir,
}

impl Test {
  pub(crate) fn args(mut self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
    assert!(self.args.is_empty());
    for arg in args {
      self.args.push(arg.as_ref().into());
    }
    self
  }

  pub(crate) fn assert_file(mut self, path: &str, expected: impl Into<String>) -> Self {
    assert!(
      self
        .files
        .insert(path.into(), Expected::string(expected))
        .is_none()
    );
    self
  }

  pub(crate) fn assert_file_regex(mut self, path: &str, pattern: &str) -> Self {
    assert!(
      self
        .files
        .insert(path.into(), Expected::regex(pattern))
        .is_none()
    );
    self
  }

  #[cfg(unix)]
  pub(crate) fn chmod(self, path: &str, mode: u32) -> Self {
    use std::os::unix::fs::PermissionsExt;
    let path = self.join(path);
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
    self
  }

  #[cfg(not(unix))]
  pub(crate) fn chmod(self, _path: &str, _mode: u32) -> Self {
    self
  }

  pub(crate) fn create_dir(self, path: &str) -> Self {
    fs::create_dir_all(self.join(path)).unwrap();
    self
  }

  pub(crate) fn current_dir(mut self, path: &str) -> Self {
    assert!(self.current_dir.is_none());
    self.current_dir = Some(path.into());
    self
  }

  pub(crate) fn data_dir(mut self, path: &str) -> Self {
    assert!(self.data_dir.is_none());
    self.data_dir = Some(path.into());
    self
  }

  pub(crate) fn env(mut self, key: &str, value: &str) -> Self {
    assert!(self.env.insert(key.into(), value.into()).is_none());
    self
  }

  #[track_caller]
  pub(crate) fn failure(self) -> Self {
    self.status(1)
  }

  fn join(&self, path: &str) -> Utf8PathBuf {
    Utf8Path::from_path(self.tempdir.path()).unwrap().join(path)
  }

  pub(crate) fn new() -> Self {
    Self::with_tempdir(
      tempfile::Builder::new()
        .prefix("filepack-test-tempdir")
        .tempdir()
        .unwrap(),
    )
  }

  pub(crate) fn path(&self) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(self.tempdir.path().into()).unwrap()
  }

  pub(crate) fn read(&self, path: &str) -> String {
    fs::read_to_string(self.join(path)).unwrap().trim().into()
  }

  pub(crate) fn read_key(&self, path: &str) -> PublicKey {
    fs::read_to_string(self.join(path))
      .unwrap()
      .trim()
      .parse()
      .unwrap()
  }

  pub(crate) fn remove_dir(self, path: &str) -> Self {
    fs::remove_dir(self.join(path)).unwrap();
    self
  }

  pub(crate) fn remove_file(self, path: &str) -> Self {
    fs::remove_file(self.join(path)).unwrap();
    self
  }

  pub(crate) fn rename(self, from: &str, to: &str) -> Self {
    fs::rename(self.join(from), self.join(to)).unwrap();
    self
  }

  #[track_caller]
  pub(crate) fn status(self, code: i32) -> Self {
    let mut command = Command::new(executable_path("filepack"));

    let current_dir = if let Some(ref subdir) = self.current_dir {
      self.join(subdir)
    } else {
      self.path()
    };

    command.current_dir(current_dir);

    let data_dir = if let Some(ref subdir) = self.data_dir {
      self.join(subdir)
    } else {
      self.path()
    };

    command.env("FILEPACK_DATA_DIR", data_dir);

    for (key, value) in &self.env {
      command.env(key, value);
    }

    command.args(&self.args);

    let child = command
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();

    if let Some(stdin) = &self.stdin {
      child
        .stdin
        .as_ref()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    }

    let output = child.wait_with_output().unwrap();

    let stdout = str::from_utf8(&output.stdout).unwrap();

    let stderr = str::from_utf8(&output.stderr).unwrap();

    if code == 0 && !output.status.success() {
      eprintln!("{stderr}");
      panic!("command failed with {}", output.status);
    }

    assert!(
      !(code != 0 && output.status.success()),
      "command unexpectedly succeeded",
    );

    assert_eq!(output.status.code(), Some(code));

    self.stderr.check(stderr, "stderr");

    self.stdout.check(stdout, "stdout");

    for (path, expected) in &self.files {
      let actual = fs::read_to_string(self.join(path)).unwrap();
      expected.check(&actual, &format!("file `{path}`"));
    }

    Self::with_tempdir(self.tempdir)
  }

  pub(crate) fn stderr(mut self, stderr: &str) -> Self {
    assert_matches!(self.stderr, Expected::Empty);
    self.stderr = Expected::String(stderr.into());
    self
  }

  pub(crate) fn stderr_path(self, stderr: &str) -> Self {
    self.stderr(&stderr.replace('/', std::path::MAIN_SEPARATOR_STR))
  }

  pub(crate) fn stderr_regex(mut self, pattern: &str) -> Self {
    assert_matches!(self.stderr, Expected::Empty);
    self.stderr = Expected::regex(pattern);
    self
  }

  pub(crate) fn stderr_regex_path(self, pattern: &str) -> Self {
    self.stderr_regex(&pattern.replace('/', std::path::MAIN_SEPARATOR_STR))
  }

  pub(crate) fn stdin(mut self, stdin: &str) -> Self {
    assert!(self.stdin.is_none());
    self.stdin = Some(stdin.into());
    self
  }

  pub(crate) fn stdout(mut self, stdout: impl AsRef<str>) -> Self {
    assert_matches!(self.stdout, Expected::Empty);
    self.stdout = Expected::String(stdout.as_ref().into());
    self
  }

  pub(crate) fn stdout_regex(mut self, pattern: &str) -> Self {
    assert_matches!(self.stdout, Expected::Empty);
    self.stdout = Expected::regex(pattern);
    self
  }

  #[track_caller]
  pub(crate) fn success(self) -> Self {
    self.status(0)
  }

  pub(crate) fn symlink(self, target: &str, link: &str) -> Self {
    let target = self.join(target);
    let link = self.join(link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(target, link).unwrap();
    self
  }

  pub(crate) fn touch(self, path: impl AsRef<Path>) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, []).unwrap();
    self
  }

  pub(crate) fn touch_non_unicode(self) -> Self {
    #[cfg(unix)]
    fn create_non_unicode_path() -> std::path::PathBuf {
      use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
      OsStr::from_bytes(&[0x80]).into()
    }

    #[cfg(windows)]
    fn create_non_unicode_path() -> std::path::PathBuf {
      use std::{ffi::OsString, os::windows::ffi::OsStringExt};
      OsString::from_wide(&[0xd800]).into()
    }

    let path = self.tempdir.path().join(create_non_unicode_path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, []).unwrap();
    self
  }

  fn with_tempdir(tempdir: tempfile::TempDir) -> Self {
    Self {
      args: Vec::new(),
      current_dir: None,
      data_dir: None,
      env: BTreeMap::new(),
      files: BTreeMap::new(),
      stderr: Expected::Empty,
      stdin: None,
      stdout: Expected::Empty,
      tempdir,
    }
  }

  pub(crate) fn write(self, path: &str, content: impl AsRef<[u8]>) -> Self {
    let path = self.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content.as_ref()).unwrap();
    self
  }
}
