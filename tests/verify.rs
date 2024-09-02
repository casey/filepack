use super::*;

#[test]
fn no_files() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{}}"#)
    .unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn extra_fields_are_not_allowed() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{},"foo":"bar"}"#)
    .unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!(
      "error: failed to deserialize manifest at `.{SEPARATOR}filepack.json`

because:
- unknown field `foo`, expected `files` at line 1 column 17\n"
    ))
    .failure();
}

#[test]
fn extraneous_file() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{}}"#)
    .unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!(
      "error: extraneous file not in manifest at `.{SEPARATOR}foo`\n"
    ))
    .failure();
}

#[test]
fn hash_mismatch() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo").write_str("bar").unwrap();

  Command::cargo_bin("filepack").unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: hash mismatch for `foo`, expected af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 but got f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d\n")
    .failure();
}
