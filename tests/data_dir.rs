use super::*;

#[cfg(unix)]
#[test]
fn default() {
  Test::new()
    .env_remove("FILEPACK_DATA_DIR")
    .env_remove("XDG_DATA_HOME")
    .env("HOME", "foo")
    .arg("info")
    .stdout(json_pretty! {
      data: "foo/.filepack",
      keychain: "foo/.filepack/keychain",
      keys: {},
    })
    .success();
}

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

#[cfg(unix)]
#[test]
fn xdg_data_home_empty() {
  Test::new()
    .env_remove("FILEPACK_DATA_DIR")
    .env("HOME", "foo")
    .env("XDG_DATA_HOME", "")
    .arg("info")
    .stdout(json_pretty! {
      data: "foo/.filepack",
      keychain: "foo/.filepack/keychain",
      keys: {},
    })
    .success();
}
