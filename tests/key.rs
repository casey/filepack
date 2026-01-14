use super::*;

#[test]
fn mismatched_key_error() {
  Test::new()
    .data_dir("foo")
    .args(["keygen"])
    .success()
    .args(["keygen"])
    .success()
    .rename("foo/keys/master.private", "keys/master.private")
    .args(["key"])
    .stderr("error: public key `master.public` doesn't match private key `master.private`\n")
    .failure();
}

#[test]
fn missing_private_key_error() {
  Test::new()
    .args(["keygen"])
    .success()
    .remove_file("keys/master.private")
    .args(["key"])
    .stderr_regex("error: private key not found: `.*master.private`\n")
    .failure();
}

#[test]
fn missing_public_key_error() {
  Test::new()
    .args(["keygen"])
    .success()
    .remove_file("keys/master.public")
    .args(["key"])
    .stderr_regex("error: public key not found: `.*master.public`\n")
    .failure();
}

#[test]
fn prints_pubkey() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .args(["key"])
    .stdout_regex("[0-9a-f]{64}\n")
    .success();

  let public_key = test.read("keys/master.public");

  test
    .args(["key"])
    .stdout(format!("{public_key}\n"))
    .success();
}

#[test]
fn prints_named_key() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .success()
    .args(["key", "--key", "deploy"])
    .stdout_regex("[0-9a-f]{64}\n")
    .success();

  let public_key = test.read("keys/deploy.public");

  test
    .args(["key", "--key", "deploy"])
    .stdout(format!("{public_key}\n"))
    .success();
}

#[test]
fn explicit_master_key() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .args(["key", "--key", "master"])
    .stdout_regex("[0-9a-f]{64}\n")
    .success();

  let public_key = test.read("keys/master.public");

  test
    .args(["key", "--key", "master"])
    .stdout(format!("{public_key}\n"))
    .success();
}

#[test]
fn nonexistent_key_error() {
  Test::new()
    .args(["keygen"])
    .success()
    .args(["key", "--key", "nonexistent"])
    .stderr_regex("error: public key not found: `.*nonexistent.public`\n")
    .failure();
}
