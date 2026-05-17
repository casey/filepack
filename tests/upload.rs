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
