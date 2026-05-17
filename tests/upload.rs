use super::*;

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
fn upload_package_uploads_files() {
  let server = Test::new()
    .serve()
    .assert_file(&format!("files/{}", Hash::bytes(b"aaa")), "aaa")
    .assert_file(&format!("files/{}", Hash::bytes(b"bbb")), "bbb")
    .assert_file(&format!("files/{}", Hash::bytes(b"ccc")), "ccc")
    .assert_file(&format!("files/{}", Hash::bytes(b"ddd")), "ddd")
    .spawn();

  Test::new()
    .write("foo", "aaa")
    .write("bar", "bbb")
    .create_dir("empty")
    .write("sub/baz", "ccc")
    .write("sub/qux", "ddd")
    .args(["create", "."])
    .success()
    .args([
      "upload",
      "--server",
      &server.address(),
      "--package",
      "manifest.filepack",
    ])
    .success();

  server.terminate().success();
}

#[test]
fn file_and_package_conflict() {
  Test::new()
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "--file",
      "foo",
      "--package",
      "bar",
    ])
    .stderr_regex(
      "error: the argument '--file <PATH>' cannot be used with '--package <PATH>'\n.*",
    )
    .status(2);
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
    .args([
      "upload",
      "--server",
      &server.address(),
      "--package",
      "manifest.filepack",
    ])
    .stderr(&format!(
      "\
mismatched file: `foo`
       manifest: {expected} (3 bytes)
           file: {actual} (3 bytes)
error: 1 mismatched file
",
    ))
    .failure();

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
      .args([
        "upload",
        "--server",
        &server.address(),
        "--package",
        "manifest.filepack",
      ])
      .success();
  }

  server.terminate().success();
}
