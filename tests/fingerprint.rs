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

  let fingerprint = "package1azljtch6chgyjc8rk7hd74wvnsqhl88zgl2g326jqpxnjlfsaaseqffcrlp";

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
