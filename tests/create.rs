use super::*;

#[test]
fn no_files() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(r#"{"files":{}}"#);

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"}}"#,
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn file_in_subdirectory() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo/bar":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"}}"#,
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
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

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!("error: path `.{SEPARATOR}�` not valid unicode\n"))
    .failure();
}

#[test]
fn symlink_error() {
  let dir = TempDir::new().unwrap();

  #[cfg(unix)]
  std::os::unix::fs::symlink("foo", dir.path().join("bar")).unwrap();

  #[cfg(windows)]
  std::os::windows::fs::symlink_file("foo", dir.path().join("bar")).unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!("error: symlink at `.{SEPARATOR}bar`\n"))
    .failure();
}

#[cfg(not(windows))]
#[test]
fn backslash_error() {
  let dir = TempDir::new().unwrap();

  dir.child("\\").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: path `\\` contains backslash\n")
    .failure();
}

#[test]
fn manifest_already_exists_error() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!(
      "error: manifest `.{SEPARATOR}filepack.json` already exists\n"
    ))
    .failure();
}