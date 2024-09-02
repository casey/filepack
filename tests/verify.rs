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
    .stderr("error: extraneous file not in filepack at `./foo`\n")
    .failure();

  Ok(())
}
