use super::*;

#[test]
fn download_retrieves_file() {
  let node = Test::new()
    .args(["server", "http://127.0.0.1:0"])
    .ready_fd()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", &node.address(), "foo"])
    .success();

  let hash = Hash::bytes(b"bar");

  Test::new()
    .args(["download", &node.address(), &hash.to_string(), "foo"])
    .assert_file("foo", "bar")
    .success();

  node.terminate().success();
}
