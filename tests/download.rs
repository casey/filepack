use super::*;

#[test]
fn download_retrieves_file() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", "--server", &server.address(), "--file", "foo"])
    .success();

  let hash = Hash::bytes(b"bar");

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--hash",
      &hash.to_string(),
      "--output",
      "foo",
    ])
    .assert_file("foo", "bar")
    .success();

  server.terminate().success();
}
