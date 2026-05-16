use {super::*, reqwest::StatusCode};

#[test]
fn reupload_succeeds() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
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
    .ready_fd()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", "--server", &server.address(), "--file", "foo"])
    .success();

  server.terminate().success();
}

#[test]
fn upload_with_wrong_hash_fails() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
    .spawn();

  let actual = Hash::bytes(b"bar");
  let expected = Hash::bytes(b"baz");

  let response = reqwest::blocking::Client::new()
    .put(format!("{}/{expected}", server.address()))
    .body("bar")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  assert_eq!(
    response.text().unwrap(),
    format!("expected upload with hash {expected} but got {actual}"),
  );

  server.terminate().success();
}
