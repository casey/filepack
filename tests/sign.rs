use super::*;

#[test]
fn appends_filename_if_argument_is_directory() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keys/master.public");

  test
    .args(["sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
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

  let public_key = test.read("keys/master.public");

  test
    .current_dir("foo")
    .args(["sign"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
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

  let a = test.read("a/keys/master.public");
  let b = test.read("b/keys/master.public");

  test
    .args(["verify", "foo", "--key", &a])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures\n")
    .success()
    .args(["verify", "foo", "--key", &b])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures\n")
    .success();
}

#[test]
fn existing_signatures_must_be_valid() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let manifest_path = test.path().join("foo/filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  manifest.signatures.insert(
    "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b"
      .parse::<PublicKey>()
      .unwrap(),
    "0".repeat(128).parse::<Signature>().unwrap(),
  );

  manifest.save(&manifest_path).unwrap();

  test
    .args(["sign", "foo/filepack.json"])
    .stderr_regex("error: invalid signature for public key `7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b`\n.*Verification equation was not satisfied\n")
    .failure();
}

#[test]
fn key_dir_insecure_permissions() {
  if !cfg!(unix) {
    return;
  }

  Test::new()
    .args(["keygen"])
    .success()
    .chmod("keys", 0o750)
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["sign", "foo/filepack.json"])
    .stderr_regex("error: keys directory `.*keys` has insecure permissions 0750\n")
    .failure();
}

#[test]
fn private_key_insecure_permissions() {
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
fn re_signing_requires_force() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keys/master.public");

  test
    .args(["sign", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success()
    .args(["sign", "foo/filepack.json"])
    .stderr(&format!(
      "error: manifest has already been signed by public key `{public_key}`\n"
    ))
    .failure()
    .args(["sign", "--force", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
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

  let public_key = test.read("keys/master.public");

  test
    .args(["verify", "foo", "--key", &public_key])
    .stderr(&format!("error: no signature found for key {public_key}\n"))
    .failure()
    .args(["sign", "foo/filepack.json"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();
}
