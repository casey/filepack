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
        created package number 1
      ",
    )
    .success();

  test
    .args(["upload", "--server", &server.address(), "bar"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 2
      ",
    )
    .success();

  for fingerprint in fingerprints {
    assert_eq!(
      reqwest::blocking::get(format!("{}/package/{fingerprint}", server.address()))
        .unwrap()
        .status(),
      StatusCode::OK,
    );
  }

  Test::new()
    .args(["delete", "--server", &server.address(), "--all"])
    .success();

  for number in 1..=2 {
    assert_eq!(
      reqwest::blocking::get(format!("{}/package/{number}", server.address()))
        .unwrap()
        .status(),
      StatusCode::NOT_FOUND,
    );
  }

  server.terminate().success();
}

#[test]
fn delete_package_not_found() {
  let server = Test::new().serve().spawn();

  Test::new()
    .args(["delete", "--server", &server.address(), "1"])
    .stderr_regex(
      "error: response from http://.* failed with status 404 Not Found: package number 1 not \
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
        created package number 1
      ",
    )
    .success();

  let url = format!("{}/package/1", server.address());

  assert_eq!(
    reqwest::blocking::get(&url).unwrap().status(),
    StatusCode::OK,
  );

  Test::new()
    .args(["delete", "--server", &server.address(), "1"])
    .success();

  assert_eq!(
    reqwest::blocking::get(&url).unwrap().status(),
    StatusCode::NOT_FOUND,
  );

  server.terminate().success();
}

#[test]
fn restricted_delete_succeeds_with_auth() {
  let server = Test::new()
    .write_keypair("master")
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
    .write_keypair("master")
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
        created package number 1
      ",
    )
    .success()
    .args([
      "delete",
      "--server",
      &server.address(),
      "--auth",
      "master",
      "1",
    ])
    .success();

  server.terminate().success();
}
