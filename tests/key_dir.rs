use super::*;

#[test]
fn directory_error() {
  Test::new()
    .create_dir("keys/subdir")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: unexpected directory in keys directory: `.*subdir`\n")
    .failure();
}

#[test]
fn unexpected_file_error() {
  Test::new()
    .write("keys/foo", "")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: unexpected file in keys directory: `.*foo`\n")
    .failure();
}

#[test]
fn type_error() {
  Test::new()
    .write("keys/master.unknown", "")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: unexpected file type `unknown` in keys directory: `.*master.unknown`\n.*")
    .failure();
}

#[test]
fn key_name_invalid_error() {
  Test::new()
    .write("keys/INVALID.public", PUBLIC_KEY)
    .write("keys/INVALID.private", PRIVATE_KEY)
    .chmod("keys", 0o700)
    .chmod("keys/INVALID.private", 0o600)
    .args(["info"])
    .stderr_regex("error: invalid key name: `INVALID`\n.*")
    .failure();
}

#[test]
fn hidden_files_are_ignored() {
  Test::new()
    .args(["keygen"])
    .success()
    .write("keys/.hidden", "")
    .write("keys/.DS_Store", "")
    .args(["info"])
    .stdout_regex(r#".*"keys": \{\n    "master":.*"#)
    .success();
}

#[test]
fn missing_private_key_error() {
  Test::new()
    .write("keys/orphan.public", PUBLIC_KEY)
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: private key not found: `.*orphan.private`\n")
    .failure();
}

#[test]
fn missing_public_key_error() {
  Test::new()
    .write("keys/orphan.private", PRIVATE_KEY)
    .chmod("keys", 0o700)
    .chmod("keys/orphan.private", 0o600)
    .args(["info"])
    .stderr_regex("error: public key not found: `.*orphan.public`\n")
    .failure();
}
