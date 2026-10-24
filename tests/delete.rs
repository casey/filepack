use super::*;

#[test]
fn delete_all() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write("foo/baz", "foo")
    .write("bar/baz", "bar")
    .args(["create", "foo"])
    .success()
    .args(["create", "bar"])
    .success();

  let fingerprints =
    ["foo", "bar"].map(|package| fingerprint(&test.path().join(package).join("manifest.filepack")));

  let test = test
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success();

  test
    .args(["upload", "--server", &server.address(), "bar"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 1
      ",
    )
    .success();

  for fingerprint in fingerprints {
    server.get(format!("/package/{fingerprint}")).send();
  }

  Test::new()
    .args(["delete", "--server", &server.address(), "--all"])
    .success();

  for number in 1..=2 {
    server
      .get(format!("/package/{number}"))
      .status(StatusCode::NOT_FOUND)
      .send();
  }

  server.terminate().success();
}

#[test]
fn delete_package_not_found() {
  let server = Test::new().serve().spawn();

  Test::new()
    .args(["delete", "--server", &server.address(), "0"])
    .stderr_regex(
      "error: response from http://.* failed with status 404 Not Found: package number 0 not \
      found\n",
    )
    .failure();

  server.terminate().success();
}

#[test]
fn delete_package_succeeds() {
  let server = Test::new().serve().spawn();

  Test::new()
    .write("foo", "bar")
    .args(["create", "."])
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success();

  server.get("/package/0").send();

  Test::new()
    .args(["delete", "--server", &server.address(), "0"])
    .success();

  server
    .get("/package/0")
    .status(StatusCode::NOT_FOUND)
    .send();

  server.terminate().success();
}

#[test]
fn restricted_delete_succeeds_with_auth() {
  let server = Test::new()
    .write_key_pair("master")
    .ready_address()
    .args([
      "serve",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--domain",
      "127.0.0.1",
      "--restrict-writes",
      "--admin-key",
      "master",
    ])
    .spawn();

  Test::new()
    .write_key_pair("master")
    .write("pkg/foo", "bar")
    .args(["create", "pkg"])
    .success()
    .args([
      "upload",
      "--server",
      &server.address(),
      "--auth",
      "master",
      "pkg",
    ])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success()
    .args([
      "delete",
      "--server",
      &server.address(),
      "--auth",
      "master",
      "0",
    ])
    .success();

  server.terminate().success();
}
