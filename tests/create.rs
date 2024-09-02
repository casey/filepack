use super::*;

#[test]
fn no_files() -> Result {
  let dir = TempDir::new()?;

  Command::cargo_bin("filepack")?
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(r#"{"files":{}}"#);

  Command::cargo_bin("filepack")?
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();

  Ok(())
}

#[test]
fn single_file() -> Result {
  let dir = TempDir::new()?;

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")?
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"}}"#,
  );

  Command::cargo_bin("filepack")?
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();

  Ok(())
}

#[test]
fn file_in_subdirectory() -> Result {
  let dir = TempDir::new()?;

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")?
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo/bar":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"}}"#,
  );

  Command::cargo_bin("filepack")?
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();

  Ok(())
}

// disable test on macos, since it does not allow non-unicode filenames
#[cfg(not(target_os = "macos"))]
#[test]
fn non_unicode_path_error() -> Result {
  use std::path::PathBuf;

  let dir = TempDir::new()?;

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

  Command::cargo_bin("filepack")?
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: path `./�` not valid unicode\n")
    .failure();

  Ok(())
}
