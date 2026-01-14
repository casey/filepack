use super::*;

#[test]
fn size() {
  Test::new()
    .write("foo/bar/baz", "hello")
    .write("foo/bar/bob", "goodbye")
    .args(["create", "."])
    .success()
    .args(["size"])
    .stdout("12\n")
    .success();
}
