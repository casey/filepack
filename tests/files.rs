use super::*;

#[test]
fn files() {
  Test::new()
    .touch("foo/bar/baz")
    .touch("foo/bar/bob")
    .arg("create")
    .success()
    .arg("files")
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

#[test]
fn with_path() {
  Test::new()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["files", "foo"])
    .stdout(json_pretty! {
      bar: {
        hash: EMPTY_HASH,
        size: 0,
      },
    })
    .success();
}
