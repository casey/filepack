use super::*;

#[test]
fn updates_manifest_with_signature() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = fs::read_to_string(dir.child("keys/master.public"))
    .unwrap()
    .trim()
    .to_owned();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(format!(
      "error: no signature found for key {public_key}\n"
    )))
    .failure();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", public_key.trim()])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn existing_signatures_must_be_valid() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let mut manifest = Manifest::load(&dir.child("foo/filepack.json"));

  manifest.signatures.insert(
    "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b".into(),
    "0".repeat(128),
  );

  manifest.store(&dir.child("foo/filepack.json"));

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .stderr(is_match("error: invalid signature for public key `7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b`\n.*Verification equation was not satisfied\n"))
    .failure();
}

#[test]
fn existing_signatures_are_preserved() {
  let dir = TempDir::new().unwrap();

  let a = dir.path().join("a");
  let b = dir.path().join("b");

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path().join("a"))
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path().join("b"))
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", &a)
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", &b)
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  let a = fs::read_to_string(a.join("keys/master.public"))
    .unwrap()
    .trim()
    .to_owned();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", &a])
    .current_dir(&dir)
    .assert()
    .success();

  let b = fs::read_to_string(b.join("keys/master.public"))
    .unwrap()
    .trim()
    .to_owned();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", &b])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn re_signing_requires_force() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = fs::read_to_string(dir.child("keys/master.public"))
    .unwrap()
    .trim()
    .to_owned();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .stderr(format!(
      "error: manifest has already been signed by public key `{public_key}`\n"
    ))
    .failure();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "--force", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();
}
