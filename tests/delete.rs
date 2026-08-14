use super::*;

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
    .write("keychain/master.public", PUBLIC_KEY)
    .write("keychain/master.private", PRIVATE_KEY)
    .chmod("keychain", 0o700)
    .chmod("keychain/master.private", 0o600)
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
    .write("keychain/master.public", PUBLIC_KEY)
    .write("keychain/master.private", PRIVATE_KEY)
    .chmod("keychain", 0o700)
    .chmod("keychain/master.private", 0o600)
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
