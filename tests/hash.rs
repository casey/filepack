use super::*;

#[test]
fn file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").write_str("foo").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["hash", "foo"])
    .current_dir(&dir)
    .assert()
    .stdout("04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9\n")
    .success();
}

#[test]
fn stdin() {
  cargo_bin_cmd!("filepack")
    .arg("hash")
    .write_stdin("foo")
    .assert()
    .stdout("04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9\n")
    .success();
}
