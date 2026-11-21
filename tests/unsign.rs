use super::*;

#[test]
fn all() {
  let test = Test::new()
    .keygen("master")
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success();

  add_unknown_signature(&test.path().join("foo/manifest.filepack"));

  test
    .args(["unsign", "--all", "foo"])
    .assert_manifest(
      "foo/manifest.filepack",
      json_pretty! {
        embedded: {},
        package: {},
        signatures: [],
      },
    )
    .success();
}

#[test]
fn keys() {
  let test = Test::new()
    .keygen("master")
    .keygen("bar")
    .keygen("baz")
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success()
    .args(["sign", "--key", "bar", "foo"])
    .success()
    .args(["sign", "--key", "baz", "foo"])
    .success();

  let master = test.read("keychain/master.public");
  let baz = test.read("keychain/baz.public");

  test
    .args(["unsign", "--key", "bar", "--key", &baz, "foo"])
    .success()
    .args(["signatures", "foo"])
    .stdout(format!("[\n  \"{master}\"\n]\n"))
    .success();
}

#[test]
fn missing_signature() {
  Test::new()
    .keygen("master")
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .args(["unsign", "--key", "master", "foo"])
    .stderr("error: no signature found for key `master`\n")
    .failure();
}
