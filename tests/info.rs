use super::*;

#[test]
fn no_keychain() {
  Test::new()
    .arg("info")
    .stdout_regex(&json_regex! {
      data_dir: ".*",
      keychain: ".*keychain",
      keys: {
      },
      version: (env!("CARGO_PKG_VERSION")),
    })
    .success();
}

#[test]
fn no_keys() {
  Test::new()
    .create_dir("keychain")
    .chmod("keychain", 0o700)
    .arg("info")
    .stdout_regex(&json_regex! {
      data_dir: ".*",
      keychain: ".*keychain",
      keys: {
      },
      version: (env!("CARGO_PKG_VERSION")),
    })
    .success();
}

#[test]
fn with_keys() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .args(["keygen", "--name", "foo"])
    .success();

  let master = test.read("keychain/master.public");
  let foo = test.read("keychain/foo.public");

  test
    .arg("info")
    .stdout_regex(&json_regex! {
      data_dir: ".*",
      keychain: ".*keychain",
      keys: {
        master: master,
        foo: foo,
      },
      version: (env!("CARGO_PKG_VERSION")),
    })
    .success();
}
