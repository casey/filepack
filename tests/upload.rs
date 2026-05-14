use super::*;

#[test]
fn upload_creates_file() {
  let node = Test::new()
    .args(["server", "127.0.0.1:0"])
    .ready_fd()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", &node.address(), "foo"])
    .success();

  node.terminate().success();
}
