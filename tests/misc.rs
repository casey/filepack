use super::*;

#[test]
fn backtraces_are_recorded_when_environment_variable_is_set() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      r"error: manifest `filepack.json` not found
",
    ))
    .failure();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("RUST_BACKTRACE", "1")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      r"error: manifest `filepack.json` not found

backtrace:
   0: .*
",
    ))
    .failure();
}
