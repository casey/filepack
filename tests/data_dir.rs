use super::*;

#[test]
fn flag() {
  Test::new()
    .args(["--data-dir", "foo", "keygen"])
    .assert_file_regex("foo/keychain/master.public", "public1a.{58}\n")
    .assert_file_regex("foo/keychain/master.private", "private1a.{58}\n")
    .success();
}

#[test]
fn xdg_data_home() {
  Test::new()
    .env_remove("FILEPACK_DATA_DIR")
    .env("XDG_DATA_HOME", "foo")
    .arg("keygen")
    .assert_file_regex("foo/filepack/keychain/master.public", "public1a.{58}\n")
    .assert_file_regex("foo/filepack/keychain/master.private", "private1a.{58}\n")
    .success();
}

#[test]
fn xdg_data_home_empty() {
  Test::new()
    .env("XDG_DATA_HOME", "")
    .arg("keygen")
    .assert_file_regex("keychain/master.public", "public1a.{58}\n")
    .assert_file_regex("keychain/master.private", "private1a.{58}\n")
    .success();
}
