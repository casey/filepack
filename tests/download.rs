use super::*;

#[test]
fn download_retrieves_file() {
  let node = Test::new()
    .args(["node", "--ready-fd", "3", "127.0.0.1:0"])
    .ready_fd()
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
