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

  let fingerprint = "package17c713f76b2c29ac6834a934011e4102a7fe23ca443c33265e77346b522458886";

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
