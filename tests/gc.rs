use super::*;

#[test]
fn gc_removes_orphaned_package_data() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write("foo", "bar")
    .args(["create", "."])
    .success();

  let fingerprint = fingerprint(&test.path().join("manifest.filepack"));

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        uploaded package number 1
      ",
    )
    .success();

  Test::new()
    .args([
      "delete",
      "--server",
      &server.address(),
      &fingerprint.to_string(),
    ])
    .success();

  Test::new()
    .args(["gc", "--server", &server.address()])
    .stderr("removed 1 revision, 1 directory, and 3 files, freeing 88 B\n")
    .success();

  Test::new()
    .args(["gc", "--server", &server.address()])
    .stderr("removed 0 revisions, 0 directories, and 0 files, freeing 0 B\n")
    .success();

  server.terminate().success();
}
