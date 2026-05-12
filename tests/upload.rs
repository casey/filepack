use super::*;

#[test]
fn upload_creates_file() {
  let node = Test::new()
    .args(["node", "--ready-fd", "3", "127.0.0.1:0"])
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .ready_fd()
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", &format!("127.0.0.1:{}", node.port()), "foo"])
    .success();

  node.terminate().success();
}
