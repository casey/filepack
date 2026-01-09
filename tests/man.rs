use super::*;

#[test]
fn man() {
  cargo_bin_cmd!("filepack")
    .arg("man")
    .assert()
    .stdout(is_match(r#".*\.TH filepack 1  "filepack \d+\.\d+\.\d+".*"#))
    .success();
}
