use super::*;

#[test]
fn fingerprint() {
  let test = Test::new()
    .touch("foo")
    .arg("create")
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
      },
    )
    .success();

  let fingerprint = "package1a6mpecnnzja3uzmdxruf87074wy778qra3yn25xuudzgjx49v3tsq9qx6vs";

  let path = test.path();

  test
    .arg("fingerprint")
    .stdout(format!("{fingerprint}\n"))
    .success()
    .args(["fingerprint", path.as_str()])
    .stdout(format!("{fingerprint}\n"))
    .success()
    .args(["fingerprint", path.join("filepack.json").as_str()])
    .stdout(format!("{fingerprint}\n"))
    .success();
}
