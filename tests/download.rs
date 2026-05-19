use super::*;

#[test]
fn download_fails_if_output_already_exists() {
  let hash = Hash::bytes(b"bar");

  Test::new()
    .write("foo", "original")
    .args([
      "download",
      "--server",
      "http://127.0.0.1:1",
      "--file",
      "--hash",
      &hash.to_string(),
      "foo",
    ])
    .assert_file("foo", "original")
    .stderr("error: `foo` already exists\n")
    .failure();
}

#[test]
fn download_fails_on_hash_mismatch() {
  let expected = Hash::bytes(b"baz");
  let actual = Hash::bytes(b"bar");

  let server = Test::new()
    .serve()
    .write(&format!("files/{expected}"), "bar")
    .spawn();

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--file",
      "--hash",
      &expected.to_string(),
      "foo",
    ])
    .stderr(&format!(
      "error: downloaded file hash mismatch: expected {expected} but got {actual}\n",
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn download_fails_with_404_when_file_missing() {
  let server = Test::new().serve().spawn();

  let hash = Hash::bytes(b"bar");

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--file",
      "--hash",
      &hash.to_string(),
      "foo",
    ])
    .stderr(&format!(
      "error: response from {}/file/{hash} failed with status 404 Not Found: file with hash f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d not found\n",
      server.address(),
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn download_package_fails_if_output_directory_already_exists() {
  Test::new()
    .create_dir("out")
    .args([
      "download",
      "--server",
      "http://example.com",
      "--hash",
      &Hash::bytes(&[]).to_string(),
      "out",
    ])
    .stderr("error: `out` already exists\n")
    .failure();
}

#[test]
fn download_package_fails_if_output_file_already_exists() {
  Test::new()
    .write("out", "")
    .args([
      "download",
      "--server",
      "http://example.com",
      "--hash",
      &Hash::bytes(&[]).to_string(),
      "out",
    ])
    .stderr("error: `out` already exists\n")
    .failure();
}

#[test]
fn download_package_fails_on_hash_mismatch() {
  let expected = Hash::bytes(b"baz");
  let actual = Hash::bytes(b"bar");

  let server = Test::new()
    .serve()
    .write(&format!("files/{expected}"), "bar")
    .spawn();

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--hash",
      &expected.to_string(),
      "out",
    ])
    .stderr(&format!(
      "error: downloaded file hash mismatch: expected {expected} but got {actual}\n",
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn download_retrieves_file() {
  let server = Test::new()
    .serve()
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
      "--file",
      "--hash",
      &hash.to_string(),
      "foo",
    ])
    .assert_file("foo", "bar")
    .success();

  server.terminate().success();
}

#[test]
fn download_retrieves_package() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write("foo", "aaa")
    .write("bar", "bbb")
    .create_dir("empty")
    .write("sub/baz", "ccc")
    .write("sub/qux", "ddd")
    .args(["create", "."])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("manifest.filepack"))).unwrap();
  let package = Hash::from(manifest.fingerprint());

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .success();

  let downloaded = Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--hash",
      &package.to_string(),
      "out",
    ])
    .assert_file("out/foo", "aaa")
    .assert_file("out/bar", "bbb")
    .assert_file("out/sub/baz", "ccc")
    .assert_file("out/sub/qux", "ddd")
    .assert_dir("out/empty")
    .success();

  assert_eq!(
    Manifest::load(Some(&downloaded.path().join("out/manifest.filepack"))).unwrap(),
    manifest,
  );

  downloaded
    .args(["verify", "out"])
    .stderr("successfully verified 4 files totaling 12 bytes\n")
    .success();

  server.terminate().success();
}

#[test]
fn server_url_must_be_http_or_https() {
  Test::new()
    .args(["download", "--server", "ftp://example.com"])
    .stderr_regex(
      "error: invalid value 'ftp://example.com' for '--server <URL>': URL scheme 'ftp' not \
       allowed, must be 'http' or 'https'\n.*",
    )
    .status(USAGE_ERROR);
}
