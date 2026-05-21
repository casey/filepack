use super::*;

#[test]
fn restricted_upload_succeeds_with_auth() {
  let server = Test::new()
    .write("keychain/master.public", PUBLIC_KEY)
    .write("keychain/master.private", PRIVATE_KEY)
    .chmod("keychain", 0o700)
    .chmod("keychain/master.private", 0o600)
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
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

  Test::new()
    .write("keychain/master.public", PUBLIC_KEY)
    .write("keychain/master.private", PRIVATE_KEY)
    .chmod("keychain", 0o700)
    .chmod("keychain/master.private", 0o600)
    .write("foo", "bar")
    .args([
      "upload",
      "--server",
      &server.address(),
      "--auth",
      "master",
      "--file",
      "foo",
    ])
    .success();

  server.terminate().success();
}

#[test]
fn reupload_package_succeeds() {
  let server = Test::new()
    .serve()
    .assert_file(&format!("files/{}", Hash::bytes(b"aaa")), "aaa")
    .assert_file(&format!("files/{}", Hash::bytes(b"bbb")), "bbb")
    .assert_file(&format!("files/{}", Hash::bytes(b"ccc")), "ccc")
    .assert_file(&format!("files/{}", Hash::bytes(b"ddd")), "ddd")
    .spawn();

  let mut test = Test::new()
    .write("foo", "aaa")
    .write("bar", "bbb")
    .create_dir("empty")
    .write("sub/baz", "ccc")
    .write("sub/qux", "ddd")
    .args(["create", "."])
    .success();

  for _ in 0..2 {
    test = test
      .args(["upload", "--server", &server.address(), "manifest.filepack"])
      .success();
  }

  server.terminate().success();
}

#[test]
fn reupload_succeeds() {
  let server = Test::new()
    .serve()
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
fn serve_admin_key_by_name() {
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

  let response = reqwest::blocking::Client::new()
    .put(format!("{}/file/{}", server.address(), Hash::bytes(b"bar")))
    .body("bar")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

  server.terminate().success();
}

#[test]
fn serve_admin_key_by_public_key() {
  let server = Test::new()
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
      PUBLIC_KEY,
    ])
    .spawn();

  let response = reqwest::blocking::Client::new()
    .put(format!("{}/file/{}", server.address(), Hash::bytes(b"bar")))
    .body("bar")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

  server.terminate().success();
}

#[test]
fn server_url_must_be_http_or_https() {
  Test::new()
    .args(["upload", "--server", "ftp://example.com"])
    .stderr_regex(
      "error: invalid value 'ftp://example.com' for '--server <URL>': URL scheme 'ftp' not \
       allowed, must be 'http' or 'https'\n.*",
    )
    .status(USAGE_ERROR);
}

#[test]
fn signatures_are_not_uploaded() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .arg("keygen")
    .success()
    .write("foo", "aaa")
    .args(["create", "."])
    .success()
    .args(["sign", "manifest.filepack"])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("manifest.filepack"))).unwrap();
  assert_eq!(manifest.signatures.len(), 1);

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
    .success();

  assert!(
    Manifest::load(Some(&downloaded.path().join("out/manifest.filepack")))
      .unwrap()
      .signatures
      .is_empty(),
  );

  server.terminate().success();
}

#[test]
fn upload_auth_requires_https() {
  Test::new()
    .args([
      "upload",
      "--server",
      "http://example.com",
      "--auth",
      "master",
      "--file",
      "foo",
    ])
    .stderr("error: authentication tokens may only be used over HTTPS or loopback\n")
    .failure();
}

#[test]
fn upload_creates_file() {
  let server = Test::new()
    .serve()
    .assert_file(&format!("files/{}", Hash::bytes(b"bar")), "bar")
    .spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", "--server", &server.address(), "--file", "foo"])
    .success();

  server.terminate().success();
}

#[test]
fn upload_package_checks_file_hashes_locally() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write("foo", "aaa")
    .args(["create", "."])
    .success()
    .write("foo", "bar");

  let expected = Hash::bytes(b"aaa");
  let actual = Hash::bytes(b"bar");

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(&format!(
      "\
mismatched file: `foo`
       manifest: {expected} (3 bytes)
           file: {actual} (3 bytes)
error: file did not match manifest entry
",
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn upload_package_fails_when_manifest_decode_fails() {
  Test::new()
    .write("manifest.filepack", "not cbor")
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "\
error: failed to decode manifest at `manifest.filepack`
       └─ expected map but found text
",
    )
    .failure();
}

#[test]
fn upload_package_fails_when_manifest_missing() {
  Test::new()
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr("error: manifest `manifest.filepack` not found\n")
    .failure();
}

#[test]
fn upload_package_fails_when_package_is_not_directory() {
  let payload_hash = Hash::bytes(b"payload");

  let mut dir_encoder = Encoder::new();
  let mut dir_map = dir_encoder.map::<u8>(2);
  dir_map.item(0, 0u8);
  dir_map.item_with(1, &(), |(), encoder| {
    let mut entries = encoder.map::<&str>(1);
    entries.item_with("package", &(), |(), encoder| {
      let mut entry = encoder.map::<u8>(3);
      entry.item(0, 0u8);
      entry.item(1, payload_hash);
      entry.item(2, 0u64);
      drop(entry);
    });
    drop(entries);
  });
  drop(dir_map);
  let dir_bytes = dir_encoder.finish();

  let root = Hash::bytes(&dir_bytes);

  let mut files = BTreeMap::new();
  files.insert(root, dir_bytes);

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u8>(3);
  archive.item(0, 0u8);
  archive.item(1, root);
  archive.item(2, &files);
  drop(archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "\
error: failed to unarchive manifest
       └─ expected archive `package` entry to be directory but found file
",
    )
    .failure();
}

#[test]
fn upload_package_fails_when_package_missing() {
  let mut dir_encoder = Encoder::new();
  let mut dir_map = dir_encoder.map::<u8>(2);
  dir_map.item(0, 0u8);
  dir_map.item(1, BTreeMap::<String, u8>::new());
  drop(dir_map);
  let dir_bytes = dir_encoder.finish();

  let root = Hash::bytes(&dir_bytes);

  let mut files = BTreeMap::new();
  files.insert(root, dir_bytes);

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u8>(3);
  archive.item(0, 0u8);
  archive.item(1, root);
  archive.item(2, &files);
  drop(archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "\
error: failed to unarchive manifest
       └─ archive missing package directory
",
    )
    .failure();
}

#[test]
fn upload_package_fails_when_root_file_missing() {
  let missing = Hash::bytes(b"missing");

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u8>(3);
  archive.item(0, 0u8);
  archive.item(1, missing);
  archive.item(2, BTreeMap::<Hash, Vec<u8>>::new());
  drop(archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(&format!(
      "\
error: failed to unarchive manifest
       └─ archive missing entry for hash {missing}
",
    ))
    .failure();
}

#[test]
fn upload_package_fails_when_root_not_directory_cbor() {
  let mut text_encoder = Encoder::new();
  text_encoder.text("not a directory");
  let junk = text_encoder.finish();
  let root = Hash::bytes(&junk);

  let mut files = BTreeMap::new();
  files.insert(root, junk);

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u8>(3);
  archive.item(0, 0u8);
  archive.item(1, root);
  archive.item(2, &files);
  drop(archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "\
error: failed to unarchive manifest
       ├─ failed to decode directory
       └─ expected map but found text
",
    )
    .failure();
}

#[test]
fn upload_package_uploads_files() {
  let server = Test::new()
    .serve()
    .assert_file(&format!("files/{}", Hash::bytes(b"aaa")), "aaa")
    .assert_file(&format!("files/{}", Hash::bytes(b"bbb")), "bbb")
    .assert_file(&format!("files/{}", Hash::bytes(b"ccc")), "ccc")
    .assert_file(&format!("files/{}", Hash::bytes(b"ddd")), "ddd")
    .assert_file_count("files", 7)
    .spawn();

  Test::new()
    .write("foo", "aaa")
    .write("bar", "bbb")
    .create_dir("empty")
    .write("sub/baz", "ccc")
    .write("sub/qux", "ddd")
    .create_dir("sub/empty")
    .args(["create", "."])
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .success();

  server.terminate().success();
}
