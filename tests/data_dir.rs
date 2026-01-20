use super::*;

#[test]
fn data_dir_flag() {
  Test::new()
    .args(["--data-dir", "custom", "keygen"])
    .assert_file_regex("custom/keychain/master.public", "public1a.{58}\n")
    .assert_file_regex("custom/keychain/master.private", "private1a.{58}\n")
    .success();
}

#[test]
fn xdg_data_home_env_var() {
  Test::new()
    .env_remove("FILEPACK_DATA_DIR")
    .env("XDG_DATA_HOME", "xdg")
    .arg("keygen")
    .assert_file_regex("xdg/filepack/keychain/master.public", "public1a.{58}\n")
    .assert_file_regex("xdg/filepack/keychain/master.private", "private1a.{58}\n")
    .success();
}
