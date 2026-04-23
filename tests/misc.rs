use super::*;

#[test]
fn backtraces_are_recorded_when_environment_variable_is_set() {
  Test::new()
    .args(["verify", "."])
    .stderr("error: manifest `manifest.filepack` not found\n")
    .failure()
    .env("RUST_BACKTRACE", "1")
    .args(["verify", "."])
    .stderr_regex(
      r"error: manifest `manifest.filepack` not found

backtrace:
   0: .*
",
    )
    .failure();
}
