use super::*;

#[test]
fn lints() {
  Test::new()
    .arg("lints")
    .stdout_regex("\\{.*\"compatibility\".*}\n")
    .success();
}
