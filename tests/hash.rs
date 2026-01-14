use super::*;

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
