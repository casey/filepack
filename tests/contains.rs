use super::*;

#[test]
fn file_found() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .args(["contains", "--file", "foo"])
    .success();
}

#[test]
fn file_missing() {
  let test = Test::new().write("foo", "bar").arg("create").success();

  test
    .write("baz", "qux")
    .args(["contains", "--file", "baz"])
    .stderr_regex("error: manifest does not contain file with hash `[0-9a-f]{64}`\n")
    .failure();
}

#[test]
fn hash_found() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .args(["contains", "--hash", EMPTY_HASH])
    .success();
}

#[test]
fn hash_missing() {
  Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["contains", "--hash", EMPTY_HASH])
    .stderr(&format!(
      "error: manifest does not contain file with hash `{EMPTY_HASH}`\n",
    ))
    .failure();
}

#[test]
fn size_mismatch() {
  Test::new()
    .touch("foo")
    .write(
      "filepack.json",
      &json! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 1,
          }
        },
        signatures: [],
      },
    )
    .args(["contains", "--file", "foo"])
    .stderr(&format!(
      "error: file with hash `{EMPTY_HASH}` has size 1 in manifest but size 0 on disk\n"
    ))
    .failure();
}

#[test]
fn target_is_required() {
  Test::new()
    .arg("create")
    .success()
    .arg("contains")
    .stderr_regex("error: the following required arguments were not provided:.*--hash.*--file.*")
    .status(2);
}
