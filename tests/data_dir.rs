use super::*;

#[test]
fn flag() {
  Test::new()
    .args(["--data-dir", "foo", "info"])
    .stdout_regex_path(&json_regex! {
      data: "foo",
      keychain: "foo/keychain",
      keys: {},
    })
    .success();
}

#[test]
fn xdg_data_home() {
  Test::new()
    .env_remove("FILEPACK_DATA_DIR")
    .env("XDG_DATA_HOME", "foo")
    .arg("info")
    .stdout_regex_path(&json_regex! {
      data: "foo/filepack",
      keychain: "foo/filepack/keychain",
      keys: {},
    })
    .success();
}

#[test]
fn xdg_data_home_empty() {
  Test::new()
    .env("XDG_DATA_HOME", "")
    .arg("info")
    .stdout_regex_path(&json_regex! {
      data: ".*filepack-test-tempdir.*",
      keychain: ".*filepack-test-tempdir.*/keychain",
      keys: {},
    })
    .success();
}
