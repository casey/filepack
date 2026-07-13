use super::*;

#[test]
fn size() {
  Test::new()
    .write("foo/bar/baz", "hello")
    .write("foo/bar/bob", "goodbye")
    .args(["create", "."])
    .success()
    .arg("size")
    .stdout(unindent(
      r#"{
        "files": 2,
        "file_size": 12,
        "directories": 2,
        "directory_size": 153
      }
      "#,
    ))
    .success();
}
