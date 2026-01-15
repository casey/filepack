use super::*;

#[test]
fn dir_permissions() {
  if !cfg!(unix) {
    return;
  }

  Test::new()
    .create_dir("keys")
    .chmod("keys", 0o750)
    .args(["keygen"])
    .stderr_regex("error: key directory `.*keys` has insecure permissions 0750\n")
    .failure();
}

#[test]
fn unexpected_directory() {
  Test::new()
    .create_dir("keys/subdir")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: unexpected directory in key directory: `.*subdir`\n")
    .failure();
}

#[test]
fn unexpected_file_no_extension() {
  Test::new()
    .touch("keys/foo")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: unexpected file in key directory: `.*foo`\n")
    .failure();
}

#[test]
fn unexpected_file_with_extension() {
  Test::new()
    .touch("keys/master.unknown")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: unexpected file in key directory: `.*master.unknown`\n")
    .failure();
}

#[test]
fn invalid_key_name() {
  Test::new()
    .write("keys/INVALID.public", PUBLIC_KEY)
    .write("keys/INVALID.private", PRIVATE_KEY)
    .chmod("keys", 0o700)
    .chmod("keys/INVALID.private", 0o600)
    .args(["info"])
    .stderr_regex("error: invalid key name: `.*INVALID.private`\n.*")
    .failure();
}

#[test]
fn private_key_permissions() {
  if !cfg!(unix) {
    return;
  }

  Test::new()
    .args(["keygen"])
    .success()
    .chmod("keys/master.private", 0o644)
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["sign", "foo/filepack.json"])
    .stderr_regex("error: private key `.*master.private` has insecure permissions 0644\n")
    .failure();
}

#[test]
fn missing_public_key_error() {
  Test::new()
    .write("keys/foo.private", PRIVATE_KEY)
    .chmod("keys", 0o700)
    .chmod("keys/foo.private", 0o600)
    .args(["info"])
    .stderr_regex("error: public key not found: `.*foo.public`\n")
    .failure();
}

#[test]
fn invalid_public_key() {
  Test::new()
    .write("keys/foo.public", "foo")
    .chmod("keys", 0o700)
    .args(["info"])
    .stderr_regex("error: invalid public key: `.*foo.public`\n.*")
    .failure();
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
fn hidden_files_are_ignored() {
  Test::new()
    .args(["keygen"])
    .success()
    .touch("keys/.hidden")
    .args(["info"])
    .stdout_regex(r#".*"keys": \{\n    "master":.*"#)
    .success();
}
