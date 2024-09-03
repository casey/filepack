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
    .stderr(
      "error: failed to deserialize manifest at `filepack.json`

because:
- unknown field `foo`, expected `files` at line 1 column 17\n",
    )
    .failure();
}

#[test]
fn extraneous_file_error() {
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
    .stderr("error: extraneous file not in manifest at `foo`\n")
    .failure();
}

#[test]
fn empty_directory_error() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{}}"#)
    .unwrap();

  dir.child("foo").create_dir_all().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: empty directory `foo`\n")
    .failure();
}

#[test]
fn multiple_empty_directories() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{}}"#)
    .unwrap();

  dir.child("foo").create_dir_all().unwrap();
  dir.child("bar").create_dir_all().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: empty directories `bar` and `foo`\n")
    .failure();
}

#[test]
fn only_leaf_empty_directory_is_reported() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{}}"#)
    .unwrap();

  dir.child("foo/bar").create_dir_all().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!("error: empty directory `foo{SEPARATOR}bar`\n"))
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

// disable test on macos, since it does not allow non-unicode filenames
#[cfg(not(target_os = "macos"))]
#[test]
fn non_unicode_path_error() {
  use std::path::PathBuf;

  let dir = TempDir::new().unwrap();

  let path: PathBuf;

  #[cfg(unix)]
  {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    path = OsStr::from_bytes(&[0x80]).into();
  };

  #[cfg(windows)]
  {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    path = OsString::from_wide(&[0xd800]).into();
  };

  dir.child(path).touch().unwrap();

  dir
    .child("filepack.json")
    .write_str(r#"{"files":{}}"#)
    .unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!("error: path `.{SEPARATOR}�` not valid unicode\n"))
    .failure();
}

#[test]
fn non_unicode_manifest_deserialize_error() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").write_binary(&[0x80]).unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(
      "error: I/O error at `filepack.json`

because:
- stream did not contain valid UTF-8
",
    )
    .failure();
}
