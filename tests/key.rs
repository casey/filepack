use super::*;

#[test]
fn default() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .arg("key")
    .stdout_regex("public1.{58}\n")
    .success();

  let public_key = test.read_public_key("keychain/master.public");

  test.arg("key").stdout(format!("{public_key}\n")).success();
}

#[test]
fn master() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .args(["key", "--key", "master"])
    .stdout_regex("public1.{58}\n")
    .success();

  let public_key = test.read_public_key("keychain/master.public");

  test
    .args(["key", "--key", "master"])
    .stdout(format!("{public_key}\n"))
    .success();
}

#[test]
fn missing() {
  Test::new()
    .arg("keygen")
    .success()
    .args(["key", "--key", "nonexistent"])
    .stderr_regex("error: public key not found: `.*nonexistent.public`\n")
    .failure();
}

#[test]
fn missing_private_key() {
  Test::new()
    .arg("keygen")
    .success()
    .remove_file("keychain/master.private")
    .arg("key")
    .stderr_regex("error: private key not found: `.*master.private`\n")
    .failure();
}

#[test]
fn missing_public_key() {
  Test::new()
    .arg("keygen")
    .success()
    .remove_file("keychain/master.public")
    .arg("key")
    .stderr_regex("error: public key not found: `.*master.public`\n")
    .failure();
}

#[test]
fn named() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .success()
    .args(["key", "--key", "deploy"])
    .stdout_regex("public1[a-z0-9]+\n")
    .success();

  let public_key = test.read_public_key("keychain/deploy.public");

  test
    .args(["key", "--key", "deploy"])
    .stdout(format!("{public_key}\n"))
    .success();
}
