use super::*;

#[test]
fn files_default() {
  Test::new()
    .touch("foo/bar/baz")
    .touch("foo/bar/bob")
    .args(["create", "."])
    .success()
    .args(["files"])
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
fn files_json() {
  Test::new()
    .touch("foo/bar/baz")
    .touch("foo/bar/bob")
    .args(["create", "."])
    .success()
    .args(["files", "--format", "json"])
    .stdout(json! {
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
fn files_tsv() {
  Test::new()
    .touch("foo/bar/baz")
    .touch("foo/bar/bob")
    .args(["create", "."])
    .success()
    .args(["files", "--format", "tsv"])
    .stdout(format!(
      "foo/bar/baz\t{EMPTY_HASH}\t0
foo/bar/bob\t{EMPTY_HASH}\t0
"
    ))
    .success();
}
