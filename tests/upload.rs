use super::*;

#[test]
fn node_creates_file() {
  let node = Test::new()
    .args(["node", "--ready-fd", "3", "127.0.0.1:0"])
    .ready_fd()
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", &format!("127.0.0.1:{}", node.port()), "foo"])
    .success();

  let node = node.terminate().success();

  let path = node
    .path()
    .join("files")
    .join(Hash::bytes(b"bar").to_string());

  assert_eq!(fs::read(path).unwrap(), b"bar");
}
