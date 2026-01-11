use super::*;

#[test]
fn size() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar/baz").write_str("hello").unwrap();

  dir.child("foo/bar/bob").write_str("goodbye").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .arg("size")
    .current_dir(&dir)
    .assert()
    .stdout("12\n")
    .success();
}
