use super::*;

#[test]
fn files() {
  Test::new()
    .touch("foo/bar/baz")
    .touch("foo/bar/bob")
    .args(["create", "."])
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
