use super::*;

#[test]
fn no_keys() {
  Test::new()
    .create_dir("keys")
    .chmod("keys", 0o700)
    .args(["info"])
    .stdout_regex(&json_regex! {
      "data-dir": ".*",
      "key-dir": ".*keys",
      keys: {
      },
    })
    .success();
}

#[test]
fn with_keys() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .args(["keygen", "--name", "foo"])
    .success();

  let master = test.read("keys/master.public");
  let foo = test.read("keys/foo.public");

  test
    .args(["info"])
    .stdout_regex(&json_regex! {
      "data-dir": ".*",
      "key-dir": ".*keys",
      keys: {
        master: master,
        foo: foo,
      },
    })
    .success();
}
