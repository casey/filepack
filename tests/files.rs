use super::*;

#[test]
fn files() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar/baz").touch().unwrap();

  dir.child("foo/bar/bob").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .arg("files")
    .current_dir(&dir)
    .assert()
    .stdout(json_pretty! {
      "foo/bar/baz": {
        hash: EMPTY_HASH,
        size: 0,
      },
      "foo/bar/bob": {
        hash: EMPTY_HASH,
        size: 0,
      },
    })
    .success();
}
