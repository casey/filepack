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
    .stderr_regex("error: keys directory `.*keys` has insecure permissions 0o40750\n")
    .failure();
}

#[test]
fn error_if_master_private_key_already_exists() {
  let test = Test::new().write("keys/master.private", "foo");

  #[cfg(unix)]
  let test = test.chmod("keys", 0o700);

  test
    .args(["keygen"])
    .stderr_regex("error: private key already exists: `.*master.private`\n")
    .failure();
}

#[test]
fn error_if_master_public_key_already_exists() {
  let test = Test::new().write("keys/master.public", "foo");

  #[cfg(unix)]
  let test = test.chmod("keys", 0o700);

  test
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

  let public_key = ed25519_dalek::VerifyingKey::from_bytes(
    &hex::decode(test.read("keys/master.public"))
      .unwrap()
      .try_into()
      .unwrap(),
  )
  .unwrap();

  let private_key = ed25519_dalek::SigningKey::from_bytes(
    &hex::decode(test.read("keys/master.private"))
      .unwrap()
      .try_into()
      .unwrap(),
  );

  assert!(!public_key.is_weak());

  assert_eq!(private_key.verifying_key(), public_key);
}
