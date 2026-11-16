use super::*;

#[test]
fn languages() {
  Test::new()
    .arg("languages")
    .stdout_regex("\\{\n.*\"en\": \"English\".*}\n")
    .success();
}
