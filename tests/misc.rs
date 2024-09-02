use super::*;

#[test]
fn backtraces_are_recorded_when_environment_variable_is_set() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(
      "error: I/O error at `./filepack.json`

because:
- No such file or directory (os error 2)
",
    )
    .failure();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("RUST_BACKTRACE", "1")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(
      predicate::str::is_match(
        r"error: I/O error at `./filepack.json`

because:
- No such file or directory \(os error 2\)

backtrace:
   0: .*
",
      )
      .unwrap(),
    )
    .failure();
}
