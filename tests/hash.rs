use super::*;

#[test]
fn assert_failure() {
  Test::new()
    .touch("foo")
    .args(["hash", "foo", "--assert", &"0".repeat(64)])
    .stderr(&format!(
      "error: file hash {} not equal to expected {}\n",
      EMPTY_HASH,
      "0".repeat(64)
    ))
    .failure();
}

#[test]
fn assert_success() {
  Test::new()
    .touch("foo")
    .args(["hash", "foo", "--assert", EMPTY_HASH])
    .stdout(format!("{EMPTY_HASH}\n"))
    .success();
}

#[test]
fn file() {
  Test::new()
    .write("foo", "foo")
    .args(["hash", "foo"])
    .stdout("04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9\n")
    .success();
}

#[test]
fn stdin() {
  Test::new()
    .arg("hash")
    .stdin("foo")
    .stdout("04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9\n")
    .success();
}
