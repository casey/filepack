use super::*;

#[test]
fn case_variant_error() {
  Test::new()
    .touch("foo/MANIFEST.FILEPACK")
    .current_dir("foo")
    .arg("fingerprint")
    .stderr("error: expected `manifest.filepack` but found case mismatch `MANIFEST.FILEPACK`\n")
    .failure()
    .args(["fingerprint", "foo"])
    .stderr("error: expected `manifest.filepack` but found case mismatch `MANIFEST.FILEPACK`\n")
    .failure();
}

#[test]
fn fingerprint() {
  let test = Test::new()
    .touch("foo")
    .arg("create")
    .assert_manifest(
      "manifest.filepack",
      json_pretty! {
        embedded: {},
        package: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
      },
    )
    .success();

  let fingerprint = "package1b91ba0d9c4b30ce661cd4434991e5938023110bf17a2649799ccbbb3048a15f0";

  let path = test.path();

  test
    .arg("fingerprint")
    .stdout(format!("{fingerprint}\n"))
    .success()
    .args(["fingerprint", path.as_str()])
    .stdout(format!("{fingerprint}\n"))
    .success()
    .args(["fingerprint", path.join("manifest.filepack").as_str()])
    .stdout(format!("{fingerprint}\n"))
    .success();
}

#[test]
fn with_path() {
  Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .args(["fingerprint", "foo"])
    .stdout("package12186aa6cefc1ae2f427b90e61a3ab91c88573f6f6719ca0188098d681b6c80ca\n")
    .success();
}
