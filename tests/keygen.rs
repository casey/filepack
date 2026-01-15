use super::*;

#[test]
fn custom_name() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .assert_file_regex("keys/deploy.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keys/deploy.private", "[0-9a-f]{64}\n")
    .success();

  let public_key = test.read_public_key("keys/deploy.public");

  let private_key = test.read_private_key("keys/deploy.private");

  assert!(!public_key.inner().is_weak());

  assert_eq!(private_key.public_key(), public_key);
}

#[test]
fn default_name() {
  let test = Test::new()
    .args(["keygen"])
    .assert_file_regex("keys/master.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keys/master.private", "[0-9a-f]{64}\n")
    .success();

  let public_key = test.read_public_key("keys/master.public");

  let private_key = test.read_private_key("keys/master.private");

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
fn key_dir_insecure_permissions() {
  if !cfg!(unix) {
    return;
  }

  Test::new()
    .create_dir("keys")
    .chmod("keys", 0o750)
    .args(["keygen"])
    .stderr_regex("error: keys directory `.*keys` has insecure permissions 0750\n")
    .failure();
}

#[test]
fn key_already_exists() {
  Test::new()
    .write("keys/master.private", PRIVATE_KEY)
    .write("keys/master.public", PUBLIC_KEY)
    .chmod("keys", 0o700)
    .chmod("keys/master.private", 0o700)
    .args(["keygen"])
    .stderr_regex("error: private key already exists: `.*master.private`\n")
    .failure();
}
