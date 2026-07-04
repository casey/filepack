use super::*;

#[test]
fn download_checks_metadata() {
  let test = Test::new()
    .touch("README.md")
    .write("metadata.yaml", "title: Foo\nreadme: README.md")
    .arg("create")
    .success();

  let metadata = fs::read(test.path().join("metadata.filemeta")).unwrap();

  let metadata_hash = Hash::bytes(&metadata);

  let directory = Directory {
    version: Version::Zero,
    entries: BTreeMap::from([(
      "metadata.filemeta".parse::<ComponentBuf>().unwrap(),
      Entry {
        ty: EntryType::File,
        hash: metadata_hash,
        size: u64::try_from(metadata.len()).unwrap(),
        total_file_size: None,
      },
    )]),
  }
  .encode_to_vec();

  let fingerprint = Fingerprint::from(Hash::bytes(&directory));

  let server = Test::new()
    .serve()
    .write(&format!("files/{}", Hash::bytes(&directory)), &directory)
    .write(&format!("files/{metadata_hash}"), &metadata)
    .spawn();

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--package",
      &fingerprint.to_string(),
      "out",
    ])
    .stderr("error: file referenced in metadata missing: `README.md`\n")
    .failure();

  server.terminate().success();
}

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
      "--package",
      &Fingerprint::from(Hash::bytes(&[])).to_string(),
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
      "--package",
      &Fingerprint::from(Hash::bytes(&[])).to_string(),
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
      "--package",
      &Fingerprint::from(expected).to_string(),
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
  let fingerprint = manifest.fingerprint();

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr("uploading 4 of 4 files\n")
    .success();

  let downloaded = Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--package",
      &fingerprint.to_string(),
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
fn download_retrieves_package_with_metadata() {
  let server = Test::new().serve().spawn();

  let mut artwork = Cursor::new(Vec::new());
  DynamicImage::new_rgb8(10, 10)
    .write_to(&mut artwork, ImageFormat::Png)
    .unwrap();

  let test = Test::new()
    .write("foo", "bar")
    .write("cover.png", artwork.into_inner())
    .write("README.md", "baz")
    .write(
      "metadata.yaml",
      "title: Foo\nartwork: cover.png\nreadme: README.md",
    )
    .args(["create", "."])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("manifest.filepack"))).unwrap();
  let fingerprint = manifest.fingerprint();

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr("uploading 5 of 5 files\n")
    .success();

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--package",
      &fingerprint.to_string(),
      "out",
    ])
    .assert_file("out/foo", "bar")
    .assert_file("out/README.md", "baz")
    .success()
    .args(["verify", "out"])
    .stderr("successfully verified 5 files totaling 250 bytes\n")
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
