use super::*;

#[test]
fn custom_name() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .assert_file_regex("keychain/deploy.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keychain/deploy.private", "[0-9a-f]{64}\n")
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
    .assert_file_regex("keychain/master.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keychain/master.private", "[0-9a-f]{64}\n")
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
      "error: invalid value '@invalid' for '--name <NAME>': invalid public key name `@invalid`\n\n\
      For more information, try '--help'.\n",
    )
    .status(2);
}

#[test]
fn key_already_exists() {
  Test::new()
    .write("keychain/master.private", PRIVATE_KEY)
    .write("keychain/master.public", PUBLIC_KEY)
    .chmod("keychain", 0o700)
    .chmod("keychain/master.private", 0o700)
    .arg("keygen")
    .stderr_regex("error: public key already exists: `.*master.public`\n")
    .failure();
}
