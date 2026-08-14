use super::*;

#[test]
fn custom_name() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .assert_file_regex("keychain/deploy.public", "public1a.{58}\n")
    .assert_file_regex("keychain/deploy.private", "private1a.{110}\n")
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
    .assert_file_regex("keychain/master.public", "public1a.{58}\n")
    .assert_file_regex("keychain/master.private", "private1a.{110}\n")
    .success();

  let public_key = test.read_public_key("keychain/master.public");

  let private_key = test.read_private_key("keychain/master.private");

  assert!(!public_key.inner().is_weak());

  assert_eq!(private_key.public_key(), public_key);
}

#[test]
fn invalid_name() {
  Test::new()
    .args(["keygen", "--name", "@invalid"])
    .stderr(
      "
        error: invalid value '@invalid' for '--name <NAME>': invalid public key name `@invalid`

        For more information, try '--help'.
      ",
    )
    .status(USAGE_ERROR);
}

#[test]
fn key_already_exists() {
  Test::new()
    .write_keypair("master")
    .arg("keygen")
    .stderr_regex("error: public key already exists: `.*master.public`\n")
    .failure();
}
