use super::*;

#[test]
fn man() {
  Test::new()
    .args(["man"])
    .stdout_regex(r#".*\.TH filepack 1  "filepack \d+\.\d+\.\d+".*"#)
    .success();
}
