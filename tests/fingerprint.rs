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
        notes: [],
      },
    )
    .success();

  let fingerprint = "package1ase89zy0tuschqfzg6ltu87devt2kt8mkr76zsuzf65kkxa4ycg8q0kds50";

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
