use super::*;

#[test]
fn error_if_key_dir_has_insecure_permissions() {
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
fn error_if_master_private_key_already_exists() {
  Test::new()
    .write("keys/master.private", "foo")
    .chmod("keys", 0o700)
    .chmod("keys/master.private", 0o700)
    .args(["keygen"])
    .stderr_regex("error: private key already exists: `.*master.private`\n")
    .failure();
}

#[test]
fn error_if_master_public_key_already_exists() {
  Test::new()
    .write("keys/master.public", "foo")
    .chmod("keys", 0o700)
    .args(["keygen"])
    .stderr_regex("error: public key already exists: `.*master.public`\n")
    .failure();
}

#[test]
fn keygen_generates_master_key_by_default() {
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
fn keygen_with_custom_name() {
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
fn error_if_named_private_key_already_exists() {
  Test::new()
    .write("keys/deploy.private", "foo")
    .chmod("keys", 0o700)
    .chmod("keys/deploy.private", 0o700)
    .args(["keygen", "--name", "deploy"])
    .stderr_regex("error: private key already exists: `.*deploy.private`\n")
    .failure();
}

#[test]
fn error_if_named_public_key_already_exists() {
  Test::new()
    .write("keys/deploy.public", "foo")
    .chmod("keys", 0o700)
    .args(["keygen", "--name", "deploy"])
    .stderr_regex("error: public key already exists: `.*deploy.public`\n")
    .failure();
}

#[test]
fn error_with_invalid_key_name() {
  Test::new()
    .args(["keygen", "--name", "@invalid"])
    .stderr(
      "error: invalid value '@invalid' for '--name <NAME>': invalid public key name `@invalid`\n\n\
      For more information, try '--help'.\n",
    )
    .status(2);
}

#[test]
fn multiple_named_keys() {
  Test::new()
    .args(["keygen", "--name", "alice"])
    .assert_file_regex("keys/alice.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keys/alice.private", "[0-9a-f]{64}\n")
    .success()
    .args(["keygen", "--name", "bob"])
    .assert_file_regex("keys/bob.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keys/bob.private", "[0-9a-f]{64}\n")
    .success();
}

#[test]
fn key_name_with_special_characters() {
  Test::new()
    .args(["keygen", "--name", "deploy-2024.prod_v1"])
    .assert_file_regex("keys/deploy-2024.prod_v1.public", "[0-9a-f]{64}\n")
    .assert_file_regex("keys/deploy-2024.prod_v1.private", "[0-9a-f]{64}\n")
    .success();
}
