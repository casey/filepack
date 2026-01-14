use super::*;

#[test]
fn assert_failure() {
  Test::new()
    .write("foo", "foo")
    .args([
      "hash",
      "foo",
      "--assert",
      "0000000000000000000000000000000000000000000000000000000000000000",
    ])
    .stderr("error: file hash 04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9 not equal to expected 0000000000000000000000000000000000000000000000000000000000000000\n")
    .failure();
}

#[test]
fn assert_success() {
  Test::new()
    .write("foo", "foo")
    .args([
      "hash",
      "foo",
      "--assert",
      "04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9",
    ])
    .stdout("04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9\n")
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
    .args(["hash"])
    .stdin("foo")
    .stdout("04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9\n")
    .success();
}
