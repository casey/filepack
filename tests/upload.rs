use super::*;

#[test]
fn node_creates_file() {
  let mut node = Test::new().args(["node", "--ready-fd", "3", "127.0.0.1:0"]);

  let mut reader = node.ready_fd();

  let node = node.spawn();

  // todo: make this reusable
  let mut port = String::new();
  reader.read_to_string(&mut port).unwrap();
  let port = port.parse::<u16>().unwrap();

  Test::new()
    .write("foo", "bar")
    .args(["upload", &format!("127.0.0.1:{port}"), "foo"])
    .success();

  let node = node.terminate().success();

  let path = node
    .path()
    .join("files")
    .join(Hash::bytes(b"bar").to_string());

  assert_eq!(fs::read(path).unwrap(), b"bar");
}
