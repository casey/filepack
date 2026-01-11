use super::*;

#[test]
fn files_default() {
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

#[test]
fn files_json() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar/baz").touch().unwrap();

  dir.child("foo/bar/bob").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["files", "--format", "json"])
    .current_dir(&dir)
    .assert()
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
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar/baz").touch().unwrap();

  dir.child("foo/bar/bob").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["files", "--format", "tsv"])
    .current_dir(&dir)
    .assert()
    .stdout(format!(
      "foo/bar/baz\t{EMPTY_HASH}\t0
foo/bar/bob\t{EMPTY_HASH}\t0
"
    ))
    .success();
}
