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

  let fingerprints = ["foo", "bar"].map(|package| {
    Manifest::load(Some(&test.path().join(package).join("manifest.filepack")))
      .unwrap()
      .fingerprint()
  });

  let test = test
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr("uploading 1 of 1 file\n")
    .success();

  test
    .args(["upload", "--server", &server.address(), "bar"])
    .stderr("uploading 1 of 1 file\n")
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

  for fingerprint in fingerprints {
    assert_eq!(
      reqwest::blocking::get(format!("{}/package/{fingerprint}", server.address()))
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

  let test = Test::new()
    .write("foo", "bar")
    .args(["create", "."])
    .success();

  let fingerprint = Manifest::load(Some(&test.path().join("manifest.filepack")))
    .unwrap()
    .fingerprint();

  Test::new()
    .args([
      "delete",
      "--server",
      &server.address(),
      &fingerprint.to_string(),
    ])
    .stderr_regex(&format!(
      "error: response from http://.* failed with status 404 Not Found: package {fingerprint} \
      not found\n"
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn delete_package_succeeds() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write("foo", "bar")
    .args(["create", "."])
    .success();

  let fingerprint = Manifest::load(Some(&test.path().join("manifest.filepack")))
    .unwrap()
    .fingerprint();

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr("uploading 1 of 1 file\n")
    .success();

  let url = format!("{}/package/{fingerprint}", server.address());

  assert_eq!(
    reqwest::blocking::get(&url).unwrap().status(),
    StatusCode::OK,
  );

  Test::new()
    .args([
      "delete",
      "--server",
      &server.address(),
      &fingerprint.to_string(),
    ])
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
      "--restrict-uploads",
      "--admin-key",
      "master",
    ])
    .spawn();

  let test = Test::new()
    .write_keypair("master")
    .write("pkg/foo", "bar")
    .args(["create", "pkg"])
    .success();

  let fingerprint = Manifest::load(Some(&test.path().join("pkg/manifest.filepack")))
    .unwrap()
    .fingerprint();

  test
    .args([
      "upload",
      "--server",
      &server.address(),
      "--auth",
      "master",
      "pkg",
    ])
    .stderr("uploading 1 of 1 file\n")
    .success()
    .args([
      "delete",
      "--server",
      &server.address(),
      "--auth",
      "master",
      &fingerprint.to_string(),
    ])
    .success();

  server.terminate().success();
}
