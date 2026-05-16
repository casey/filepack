use super::*;

#[test]
fn download_fails_if_output_already_exists() {
  let hash = Hash::bytes(b"bar");

  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
    .write(&format!("files/{hash}"), "bar")
    .spawn();

  Test::new()
    .write("foo", "original")
    .args([
      "download",
      "--server",
      &server.address(),
      "--hash",
      &hash.to_string(),
      "--output",
      "foo",
    ])
    .assert_file("foo", "original")
    .stderr(
      "error: I/O error at `foo`
       └─ File exists (os error 17)\n",
    )
    .failure();

  server.terminate().success();
}

#[test]
fn download_fails_on_hash_mismatch() {
  let expected = Hash::bytes(b"baz");
  let actual = Hash::bytes(b"bar");

  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
    .write(&format!("files/{expected}"), "bar")
    .spawn();

  Test::new()
    .args([
      "download",
      "--server",
      &server.address(),
      "--hash",
      &expected.to_string(),
      "--output",
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
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
    .spawn();

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
    .stderr(&format!(
      "error: response from {}/{hash} failed with status 404 Not Found: file with hash f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d not found\n",
      server.address(),
    ))
    .failure();

  server.terminate().success();
}

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
