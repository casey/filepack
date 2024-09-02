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
