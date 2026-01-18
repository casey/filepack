use super::*;

#[test]
fn found() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .args(["contains", "--hash", EMPTY_HASH])
    .success();
}

#[test]
fn missing() {
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
