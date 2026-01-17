use super::*;

#[test]
fn lints() {
  Test::new()
    .arg("lints")
    .stdout(
      r#"{
  "compatibility": [
    "case-conflict",
    "filename-length",
    "windows-leading-space",
    "windows-reserved-character",
    "windows-reserved-filename",
    "windows-trailing-period",
    "windows-trailing-space"
  ],
  "distribution": [
    "case-conflict",
    "filename-length",
    "junk",
    "windows-leading-space",
    "windows-reserved-character",
    "windows-reserved-filename",
    "windows-trailing-period",
    "windows-trailing-space"
  ],
  "junk": [
    "junk"
  ]
}"#
        .to_owned()
        + "\n",
    )
    .success();
}
