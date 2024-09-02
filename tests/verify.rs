use super::*;

#[test]
fn no_files() -> Result {
  let dir = TempDir::new()?;

  dir.child("filepack.json").write_str(r#"{"files":{}}"#)?;

  Command::cargo_bin("filepack")?
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();

  Ok(())
}

#[test]
fn extraneous_file() -> Result {
  let dir = TempDir::new()?;

  dir.child("filepack.json").write_str(r#"{"files":{}}"#)?;

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")?
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!(
      "error: extraneous file not in filepack at `.{SEPARATOR}foo`\n"
    ))
    .failure();

  Ok(())
}

#[test]
fn hash_mismatch() -> Result {
  let dir = TempDir::new()?;

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")?
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo").write_str("bar").unwrap();

  Command::cargo_bin("filepack")?
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: hash mismatch for `foo`, expected af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 but got f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d\n")
    .failure();

  Ok(())
}
