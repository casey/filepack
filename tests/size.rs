use super::*;

#[test]
fn size() {
  Test::new()
    .write("foo/bar/baz", "hello")
    .write("foo/bar/bob", "goodbye")
    .arg("create")
    .success()
    .arg("size")
    .stdout(
      r#"
        {
          "files": 2,
          "file_size": 12,
          "directories": 2,
          "directory_size": 147
        }
      "#,
    )
    .success();
}

#[test]
fn with_path() {
  Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .args(["size", "foo"])
    .stdout(
      r#"
        {
          "files": 0,
          "file_size": 0,
          "directories": 0,
          "directory_size": 0
        }
      "#,
    )
    .success();
}
