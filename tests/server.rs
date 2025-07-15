use super::*;

#[test]
fn invalid_archives_trigger_an_error() {
  let dir = TempDir::new().unwrap();

  dir.child("invalid.filepack").write_str("invalid").unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["server", "--port", "0", dir.path().to_str().unwrap()])
    .assert()
    .failure()
    .stderr(is_match(
      "error: failed to load archive at `.*invalid.filepack`.*",
    ));
}

#[test]
fn listens_on_correct_port() {
  let dir = TempDir::new().unwrap();

  let port = free_port();

  let mut child = std::process::Command::new(executable_path("filepack"))
    .args(["server", "--port", &port.to_string()])
    .arg(dir.path())
    .spawn()
    .unwrap();

  let start = Instant::now();

  loop {
    if let Ok(response) = reqwest::blocking::get(format!("http://127.0.0.1:{port}")) {
      if response.status().is_success() {
        break;
      }
    }

    if start.elapsed() > Duration::from_secs(60) {
      panic!("server failed to start");
    }

    thread::sleep(Duration::from_millis(50));
  }

  child.kill().unwrap();
}
