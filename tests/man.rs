use super::*;

#[test]
fn man() {
  Command::cargo_bin("filepack")
    .unwrap()
    .arg("man")
    .assert()
    .stdout(is_match(r#".*\.TH filepack 1  "filepack \d+\.\d+\.\d+".*"#))
    .success();
}
