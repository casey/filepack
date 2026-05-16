use super::*;

#[test]
fn reupload_succeeds() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_address()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  for _ in 0..2 {
    Test::new()
      .write("foo", "bar")
      .args(["upload", "--server", &server.address(), "--file", "foo"])
      .success();
  }

  server.terminate().success();
}

#[test]
fn upload_creates_file() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_address()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", "--server", &server.address(), "--file", "foo"])
    .success();

  server.terminate().success();
}
