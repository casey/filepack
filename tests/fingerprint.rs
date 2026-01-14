use super::*;

#[test]
fn fingerprint() {
  let test = Test::new()
    .touch("foo")
    .args(["create"])
    .assert_file(
      "filepack.json",
      json! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: {},
      },
    )
    .success();

  let fingerprint = "864e5111ebe431702448d7d7c3f9b962d5659f761fb4287049d52d6376a4c20e";

  let path = test.path();

  test
    .args(["fingerprint"])
    .stdout(format!("{fingerprint}\n"))
    .success()
    .args(["fingerprint", path.as_str()])
    .stdout(format!("{fingerprint}\n"))
    .success()
    .args(["fingerprint", path.join("filepack.json").as_str()])
    .stdout(format!("{fingerprint}\n"))
    .success();
}
