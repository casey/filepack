use super::*;

#[test]
fn ready_fd_must_not_conflict_with_standard_streams() {
  Test::new()
    .args(["serve", "--address", "127.0.0.1:0", "--ready-fd", "2"])
    .stderr_regex(
      "error: invalid value '2' for '--ready-fd <READY_FD>': 2 is not in 3\\.\\.=2147483647\n\n.*",
    )
    .status(2);
}
