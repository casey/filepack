use super::*;

#[test]
fn default() {
  Test::new()
    .arg("languages")
    .stdout_regex("\\{\n.*\"en\": \"English\".*}\n")
    .success();
}

#[test]
fn json() {
  Test::new()
    .args(["languages", "--format", "json"])
    .stdout_regex("\\{.*\"en\":\"English\".*}\n")
    .success();
}
