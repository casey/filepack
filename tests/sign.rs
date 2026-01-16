use super::*;

#[test]
fn appends_filename_if_argument_is_directory() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn defaults_to_current_directory() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .current_dir("foo")
    .args(["sign"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn existing_signatures_are_preserved() {
  let test = Test::new()
    .data_dir("a")
    .args(["keygen"])
    .success()
    .data_dir("b")
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .data_dir("a")
    .args(["sign", "foo/filepack.json"])
    .success()
    .data_dir("b")
    .args(["sign", "foo/filepack.json"])
    .success();

  let a = test.read("a/keychain/master.public");
  let b = test.read("b/keychain/master.public");

  test
    .args(["verify", "foo", "--key", &a])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures across 1 note\n")
    .success()
    .args(["verify", "foo", "--key", &b])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures across 1 note\n")
    .success();
}

#[test]
fn mismatched_key() {
  Test::new()
    .data_dir("foo")
    .args(["keygen"])
    .success()
    .args(["keygen"])
    .success()
    .rename("foo/keychain/master.private", "keychain/master.private")
    .create_dir("bar")
    .args(["create", "bar"])
    .success()
    .args(["sign", "bar"])
    .stderr("error: public key `master.public` doesn't match private key `master.private`\n")
    .failure();
}

#[test]
fn named() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/deploy.public");

  test
    .args(["sign", "--key", "deploy", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn re_signing_requires_force() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["sign", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success()
    .args(["sign", "foo/filepack.json"])
    .stderr(&format!(
      "error: manifest has already been signed by key `{public_key}`\n"
    ))
    .failure()
    .args(["sign", "--force", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn updates_manifest_with_signature() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["verify", "foo", "--key", &public_key])
    .stderr(&format!(
      "error: no signature found for key `{public_key}`\n"
    ))
    .failure()
    .args(["sign", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}
