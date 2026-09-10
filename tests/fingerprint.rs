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

  let fingerprint = "package1a03cn7a4jc2dvdq62jdqpreqs9fl7y09yg0pnye08wdrt2gj93zrqume76y";

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
