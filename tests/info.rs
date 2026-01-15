use super::*;

#[test]
fn no_key_dir() {
  Test::new()
    .args(["info"])
    .stdout_regex(&json_regex! {
      "data-dir": ".*",
      "key-dir": ".*keychain",
      keys: {
      },
    })
    .success();
}

#[test]
fn no_keys() {
  Test::new()
    .create_dir("keychain")
    .chmod("keychain", 0o700)
    .args(["info"])
    .stdout_regex(&json_regex! {
      "data-dir": ".*",
      "key-dir": ".*keychain",
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

  let master = test.read("keychain/master.public");
  let foo = test.read("keychain/foo.public");

  test
    .args(["info"])
    .stdout_regex(&json_regex! {
      "data-dir": ".*",
      "key-dir": ".*keychain",
      keys: {
        master: master,
        foo: foo,
      },
    })
    .success();
}
