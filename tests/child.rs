use {
  super::*,
  axum::{body, response::IntoResponse},
  std::sync::LazyLock,
  tokio::runtime::Runtime,
};

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

pub(crate) struct Child {
  child: Option<std::process::Child>,
  port: Option<u16>,
  test: Option<Test>,
}

impl Child {
  pub(crate) fn address(&self) -> String {
    format!("http://127.0.0.1:{}", self.port.unwrap())
  }

  #[track_caller]
  pub(crate) fn assert_response(&self, path: &str, expected: impl IntoResponse) {
    let url = format!("{}{path}", self.address());

    let actual = reqwest::blocking::get(&url).unwrap();
    let actual_status = actual.status();
    let actual_headers = actual.headers().clone();
    let actual_body = actual.bytes().unwrap();

    let (parts, body) = expected.into_response().into_parts();
    let expected_body = RUNTIME.block_on(body::to_bytes(body, usize::MAX)).unwrap();

    assert_eq!(actual_status, parts.status);

    for (name, value) in &parts.headers {
      assert_eq!(
        actual_headers.get(name),
        Some(value),
        "header `{name}` mismatch",
      );
    }

    assert_eq!(actual_body, expected_body);
  }

  pub(crate) fn new(child: std::process::Child, port: Option<u16>, test: Test) -> Self {
    Self {
      child: Some(child),
      port,
      test: Some(test),
    }
  }

  #[track_caller]
  pub(crate) fn success(mut self) -> Test {
    let (child, test) = self.take();

    let output = child.wait_with_output().unwrap();

    test.status_with_output(0, output)
  }

  pub(crate) fn take(&mut self) -> (std::process::Child, Test) {
    (self.child.take().unwrap(), self.test.take().unwrap())
  }

  #[cfg(unix)]
  pub(crate) fn terminate(self) -> Self {
    let pid = self.child.as_ref().unwrap().id();

    let result = unsafe { libc::kill(pid.try_into().unwrap(), libc::SIGTERM) };

    assert_eq!(result, 0, "{}", std::io::Error::last_os_error());

    self
  }

  #[cfg(windows)]
  pub(crate) fn terminate(self) -> Self {
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, GenerateConsoleCtrlEvent};

    let pid = self.child.as_ref().unwrap().id();

    let result = unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) };

    assert_ne!(result, 0, "{}", std::io::Error::last_os_error());

    self
  }
}

impl Drop for Child {
  fn drop(&mut self) {
    if let Some(child) = self.child.as_mut() {
      child.kill().unwrap();
      child.wait().unwrap();
    }
  }
}
