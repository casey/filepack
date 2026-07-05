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

  let fingerprint = "package1akzf8204dnnly606mjw376rx2xslf8m2tptptrmk2h7vtxaplqs9qpjvqax";

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
