use super::*;

#[test]
fn gc_removes_orphaned_package_data() {
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
    .stdout_regex("removed 1 directory and 2 files, freeing [0-9]+ bytes\n")
    .success();

  Test::new()
    .args(["gc", "--server", &server.address()])
    .stdout("removed 0 directories and 0 files, freeing 0 bytes\n")
    .success();

  server.terminate().success();
}
