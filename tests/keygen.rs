use super::*;

#[test]
fn custom_name() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .assert_file_regex("keychain/deploy.public", "public1[0-9a-f]{64}\n")
    .assert_file_regex("keychain/deploy.private", "private1[0-9a-f]{128}\n")
    .success();

  let public_key = test.read_public_key("keychain/deploy.public");

  let private_key = test.read_private_key("keychain/deploy.private");

  assert!(!public_key.inner().is_weak());

  assert_eq!(private_key.public_key(), public_key);
}

#[test]
fn default_name() {
  let test = Test::new()
    .arg("keygen")
    .assert_file_regex("keychain/master.public", "public1[0-9a-f]{64}\n")
    .assert_file_regex("keychain/master.private", "private1[0-9a-f]{128}\n")
    .success();

  let public_key = test.read_public_key("keychain/master.public");

  let private_key = test.read_private_key("keychain/master.private");

  assert!(!public_key.inner().is_weak());

  assert_eq!(private_key.public_key(), public_key);
}

#[test]
fn key_already_exists() {
  Test::new()
    .write_keypair("master")
    .arg("keygen")
    .stderr_regex("error: public key already exists: `.*master.public`\n")
    .failure();
}
