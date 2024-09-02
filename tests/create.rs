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
