use super::*;

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

  let fingerprint = "package1dc801b91185c3bddf3007eb3f015905022359a229455b93aa8b5281dd89dca2e";

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
